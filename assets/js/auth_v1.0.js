import { apiCall } from '/assets/js/api_call_v1.0.js';

const signUpButton = document.getElementById('signUp');
const signInButton = document.getElementById('signIn');
const signInTriggerButton = document.getElementById('sign-in-trigger-btn');
const container = document.getElementById('container');

signUpButton.addEventListener('click', () => {
    container.classList.add("right-panel-active");
});

signInButton.addEventListener('click', () => {
    container.classList.remove("right-panel-active");
});

signInTriggerButton.addEventListener('click', async (event) => {
    event.preventDefault();

    let email = signInTriggerButton.parentElement.querySelector('input[type="email"]').value;

    if (!email) {
        alert("Please enter your email");
        return;
    }

    let password = signInTriggerButton.parentElement.querySelector('input[type="password"]').value;

    if (!password) {
        alert("Please enter your password");
        return;
    }

    let response = await apiCall({
        method: "POST",
        apiUrl: "/api/auth/sign-in",
        reqBody: {
            email_or_username: email,
            password: password
        }
    })

    console.log(response);

    if (response.ok) {
        let payload = response.data.auth_payload;

        if (payload.role != "Administrator") {
            alert("You are not an admin");
            return;
        }

        localStorage.setItem("access_token", payload.access_token);
        localStorage.setItem("refresh_token", payload.refresh_token);
        localStorage.setItem("user_id", payload.user_id);
        localStorage.setItem("role", payload.role);

        location.href = "/dashboard";
    }
});