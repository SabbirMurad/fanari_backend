/**
 * This is a wrapper function for making API calls to the server. It takes in an object
 * with all the necessary information to make the API call.
 * 
 * @param {Object} args - An object containing all the necessary information for the API call
 * 
 * @param {string} args.origin - The origin of the API call. This is used to make the actual
 * API call and is also used to check if the API is reachable
 * or not. If not provided, it defaults to the value of the
 * WEB_ORIGIN environment variable, or "" if it doesn't exist.
 * 
 * @param {string} args.method - The HTTP method to use for the API call. This is usually
 * either "GET" or "POST". If not provided, it defaults to "GET".
 * 
 * @param {Object} args.headers - An object containing additional HTTP headers to send
 * with the API call. This is used to pass things like Authorization
 * headers or Content-Type headers.
 * 
 * @param {Object} args.postData - The data to send in the request body, if the method is
 * "POST". This is usually an object, but can be of any type.
 * 
 * @param {string} args.apiUrl - The URL path of the API endpoint. This is the path that is
 * appended to the origin to make the actual API call.
 * 
 * @param {number} args.retryLimit - The number of times to retry the API call if it fails.
 * This is useful in cases where the API may be temporarily
 * unavailable. If not provided, it defaults to 5.
 * 
 * @return {Object} - Returns an object with the status of the API call. If the call was
 * successful, it will have the following shape:
 * {
 *   ok: true,
 *   status_code: <HTTP status code>,
 *   data: <parsed JSON response data>,
 *   message: <message from server>
 * }
 * If the call was not successful, it will have one of the following shapes:
 * {
 *   ok: false,
 *   status_code: <HTTP status code>,
 *   message: <error message from server>
 * }
 */
async function apiCall(args) {
    // Set the default values for the arguments
    const origin = args.origin || window.location.origin || "";
    const method = args.method || "GET";
    const headers = args.headers || {};
    const reqBody = args.reqBody || {};
    const apiUrl = args.apiUrl;
    let retryLimit;

    if (args.retryLimit === undefined || args.retryLimit === null) {
        retryLimit = 1;
    } else {
        retryLimit = parseInt(args.retryLimit);
    }

    // Create the request object that will be sent
    const reqObj = {
        method,
        headers: {
            // Set the default headers that will be sent with the request
            Accept: "application/json",
            "Content-Type": "application/json",
            // If the user is logged in, add the Authorization header
            // with the access token
            ...(localStorage.getItem("access_token") ? {
                Authorization: `Bearer ${localStorage.getItem("access_token")}`
            } : {}),
            // Add the custom headers provided by the user
            ...headers,
        },
    };

    // If the method is "POST", add the request body
    if (method === "POST") reqObj.body = JSON.stringify(reqBody);

    let response;
    try {
        response = await fetch(`${origin}${apiUrl}`, reqObj);
    } catch (error) {
        console.error(error);
        return {
            ok: false,
            message: "Failed to connect to the server, please try again later or contact support team.",
        };
    }

    if (response.ok) {
        let result;
        try {
            result = await response.json();
        } catch (error) {
            console.error(error);
            return {
                ok: false,
                status_code: response.status,
                message: "Error Parsing Response"
            };
        }

        return {
            ok: true,
            status_code: response.status,
            data: result,
            message: result?.message || null,
        };
    }

    if (response.status === 401 && retryLimit > 0) {
        retryLimit--;
        const response2 = await getNewAccessToken();

        if (!response2.ok) {
            if (response2.status_code === 401) {
                setTimeout(() => { window.location.href = "/sign-out" }, 2000);
            }

            return {
                ok: false,
                status_code: response2.status,
                data: null,
                message: response2.message
            };
        }

        return await apiCall({ origin, method, headers, postData, apiUrl, retryLimit });
    }

    let message;
    try {
        const result = await response.json();
        message = result.message || "Response Not Okay";
    } catch (error) {
        console.error(error);
        if (response.status === 400) {
            message = "Bad request, Check the post data";
        } else if (response.status === 404) {
            message = "Not found, Check the api route";
        } else {
            message = "Response Not Okay";
        }
    }

    return {
        ok: false,
        status_code: response.status,
        message
    };
}


/**
 * This function is responsible for getting a new access token using the 
 * refresh token from localStorage. It uses the api endpoint 
 * /api/account/access-token/new to get a new access token. The function 
 * makes a POST request to this endpoint with the refresh token, user id 
 * and role stored in localStorage as the request body. It then parses the 
 * response and sets the new access token in localStorage. If an error 
 * occurs, the function logs the response to the console and returns false.
 * 
 * @returns {boolean|object} Returns false if an error occurs or the 
 * response object from the api call if everything goes well.
 */
async function getNewAccessToken() {
    const response = await fetch(`${window.WEB_ORIGIN || ""}/api/auth/refresh`, {
        method: 'POST', // Use POST method to send a new access token request
        headers: {
            // Set content type to application/json
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            refresh_token: localStorage.getItem('refresh_token'),
            user_id: localStorage.getItem('user_id'),
            role: localStorage.getItem('role')
        })
    });

    if (!response.ok) { // If the response is not okay
        let message;
        try {
            const result = await response.json();
            message = result.message || "Response Not Okay";
        } catch (error) {
            console.error(error);
            if (response.status === 400) {
                message = "Bad request, Check the post data";
            } else if (response.status === 404) {
                message = "Not found, Check the api route";
            } else {
                message = "Response Not Okay";
            }
        }

        return {
            ok: false,
            status_code: response.status,
            message: message
        }; // Return false indicating an error occurred
    }

    const result = await response.json();

    let access_token = result.access_token;
    localStorage.setItem('access_token', access_token);

    let access_token_valid_till = result.access_token_valid_till;
    localStorage.setItem('access_token_valid_till', access_token_valid_till);

    return {
        ok: true,
        status_code: 200,
        message: 'All okay'
    };
}

export { apiCall }
