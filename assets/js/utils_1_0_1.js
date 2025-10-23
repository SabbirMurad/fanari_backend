/**  Use this file as module (export/import) */
/**
 * This function takes a UNIX timestamp as an argument,
 * and returns a string with the date in a human-readable format.
 *
 * The function first converts the timestamp to a JavaScript Date object,
 * which has methods to extract the month, day, and year.
 *
 * It then uses an array of month names to get the name of the month,
 * based on the zero-based index of the month in the Date object.
 *
 * The function then uses the month, day, and year to create a string
 * in the format "DAY MONTH YEAR".
 *
 * @param {number} timestamp - A UNIX timestamp in milliseconds.
 * @return {string} A string representing the date in a human-readable format.
 */
function prettyDate(timestamp) {
    const date = new Date(timestamp)
    const months = [
        'Jan',
        'Feb',
        'Mar',
        'Apr',
        'May',
        'Jun',
        'Jul',
        'Aug',
        'Sep',
        'Oct',
        'Nov',
        'Dec'
    ]
    const month = months[date.getMonth()]
    const day = date.getDate()
    const year = date.getFullYear()
    return `${day} ${month} ${year}`
}



/**
 * Converts an ArrayBuffer to a base64 encoded string.
 *
 * @param {ArrayBuffer} buffer The ArrayBuffer to convert.
 * @return {string} The base64 encoded string.
 */
function arrayBufferToBase64(buffer) {
    const bytes = new Uint8Array(buffer);
    const binaryString = String.fromCharCode(...bytes);
    const base64 = btoa(binaryString);
    return `data:image/png;base64,${base64}`;
}

/**
 * Formats a number with a magnitude suffix (K, M, B, T) based on its
 * magnitude. For example, the number 1234567 becomes "1.2M".
 *
 * @param {number} value The number to format.
 * @returns {string} The formatted number with a magnitude suffix.
 * @throws {Error} If the value is null, undefined, or not a number.
 * @throws {Error} If value does not have a toFixed method.
 */
function formatNumberMagnitude(value) {
    const SUFFIXES = ["", "K", "M", "B", "T"];
    let MAGNITUDE = 0;

    // Check if value is not null, undefined, or NaN
    if (value === null || value === undefined || Number.isNaN(value)) {
        throw new Error("formatNumberMagnitude: value is null or undefined or not a number");
    }

    // Find the appropriate magnitude for the number
    while (value >= 1000) {
        value /= 1000;
        MAGNITUDE++;
    }

    // Make sure value has a toFixed method (all numbers do in JS)
    if (value.toFixed === undefined) {
        throw new Error("Value does not have a toFixed method");
    }

    // Format the number with one decimal place and the appropriate suffix
    return `${value.toFixed(1)}${SUFFIXES[MAGNITUDE]}`;
}


/**
 * Formats a monetary amount into a human-readable string.
 *
 * For example, if the amount is 1234.56, it will return the string "1,234.56".
 *
 * @param {number} amount The monetary amount to format.
 * @returns {string} The formatted monetary amount.
 * @throws {Error} If amount is null, undefined, or not a number.
 */
function formatMoney(amount) {
    // Check if amount is not null, undefined, or NaN
    if (amount === null || amount === undefined || Number.isNaN(amount)) {
        throw new Error("formatMoney: amount is null or undefined or not a number");
    }

    // Convert amount to a string and split it into whole and fraction parts
    const [whole, fraction] = String(amount).split('.');

    // Insert commas into the whole part of the number to make it more readable
    const formattedWhole = whole.replace(/\B(?=(\d{3})+(?!\d))/g, ',');

    // Ensure fraction part is 2 digits long (00)
    const formattedFraction = (fraction || '00').slice(0, 2);

    // Concatenate whole and fraction parts with a '.' in between
    return `${formattedWhole}.${formattedFraction}`;
}

/**
* This function checks if the inputText contains any of the banned words
* and returns an error message if it does, otherwise it just returns
* the original inputText.
*
* @param {string} inputText - The text that needs to be checked
* @returns {string} inputText or an error message if the inputText violates our policy
*/
function checkForProfanity(inputText) {
    /**
    * An array of banned words that we don't want our users to use
    * @type {string[]}
    */
    const bannedWords = [
        'fuck', 'boobs', 'ass', 'sex', 'shit', 'bitch', 'dick',
        'pussy', 'dildo', 'asshole', 'cunt', 'faggot'
    ];

    for (let i = 0; i < bannedWords.length; i++) {
        if (inputText.includes(bannedWords[i])) {
            return Error(`The word ${bannedWords[i]} violates our policy.`);
        }
    }

    return inputText;
}

/**
 * Converts a dash-case string to camelCase.
 *
 * @param {string} str - The dash-case string to convert.
 * @return {string} The camelCase version of the input string.
 */
function dashCaseToCamelCase(str) {
    return str.tabValue.split("-").map((word) => {
        return word.charAt(0).toUpperCase() + word.slice(1)
    }).join("")
}

window.addEventListener("load", () => {
    if (!window.company) {
        window.company = new Object;
    }

    window.company.prettyDate = prettyDate;
    window.company.formatNumberMagnitude = formatNumberMagnitude;
    window.company.formatMoney = formatMoney;
    window.company.checkForProfanity = checkForProfanity;
    window.company.dashCaseToCamelCase = dashCaseToCamelCase;
})