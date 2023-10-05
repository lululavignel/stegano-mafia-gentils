/**
 * Client-side code for handling username validation functionality.
 * It establishes a socket connection with the server, checks if a username
 * is registered and validated, and redirects the user based on the response.
 */

$(function () {
    let socket = io();

    const usernameButton = document.getElementById('usernameButton');

    if (usernameButton) {
        usernameButton.addEventListener('click', () => {
            username = document.getElementById('username').value;
            socket.emit('check_username_registered_validated', (username));
        });

        socket.on('username_known', username => {
            window.location.href = '/connection.html?username=' + username;
        });

        socket.on('username_unknown', username => {
            window.location.href = '/register.html?username=' + username;
        });
    }
});