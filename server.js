// TODO: on veut que lorsqu'on envoit une image une interface se présente à l'utilisateur lui proposant d'utiliser un algorithme d'encryptage, l'autre mafieux pourra faire tourner tous les algorithmes (côtés serveur pour essayer de décrypter un message d'une image) et le serveur lui renverrait le résultat de toutes ces decryptages.

const fs = require('fs');
const he = require('he');
const cryptico = require('cryptico');
const CryptoJS = require('crypto-js');
const bodyParser = require('body-parser');
const express = require('express');
const rateLimit = require('express-rate-limit');
const app = express();
const path = require('path');
const server = require('http').createServer(app);
const io = require('socket.io')(server);
const port = process.env.PORT || 3000;
const imageFolder = 'images';

const Rooms = require('./room.js');
const Users = require('./user.js');

// Sever's key pair generated at every launch
const serverKeyPair = generateServerKeyPair();
const serverPublicKey = cryptico.publicKeyString(serverKeyPair);

// Used to regroup chunks when user send a data package way too big
let userDataRegistrationChunks = {};
let userDataConnectionChunks = {};

// Load application config/state
loadUsersData(), loadRoomsData(), loadPrivateDirectMessagesData(), loadPublicMessagesData();

// Implement the rate limiter to 100 requests per minute per IP address
const limiter = rateLimit({
    windowMs: 60 * 1000,
    max: 100,
});
app.use(limiter);

// Start server
server.listen(port, () => {
    console.log('Server listening on port %d', port);
});

// Use body-parser middleware to parse request body
app.use(bodyParser.urlencoded({ extended: true }));

// Routing for client-side files
app.use(express.static(path.join(__dirname, 'public/pages')));

// Set the proper MIME type for JavaScript files
app.use('/clients', express.static(path.join(__dirname, 'public/clients'), {
    setHeaders: (res, filePath) => {
        if (path.extname(filePath) === '.js') {
            res.setHeader('Content-Type', 'text/javascript');
        }
    },
}));

// Define a route to serve PNG images with custom headers
app.use('/images', express.static(path.join(__dirname, 'public/images'), {
    setHeaders: (res, filePath) => {
        if (path.extname(filePath) === '.png') {
            res.setHeader('Content-Type', 'image/png');
        }
    },
}));

// Set the proper MIME type for WebAssembly files
app.use('/pkg', express.static(path.join(__dirname, 'pkg'), {
    setHeaders: (res, filePath) => {
        // Check if the file has a .wasm extension
        if (path.extname(filePath) === '.wasm') {
            res.setHeader('Content-Type', 'application/wasm');
        }
    },
}));

// Set the proper MIME type for CSS files
app.use('/styles', express.static(path.join(__dirname, 'public/styles'), {
    setHeaders: (res, filePath) => {
        if (path.extname(filePath) === '.css') {
            res.setHeader('Content-Type', 'text/css');
        }
    },
}));

////////////////////////////////////////////////////////////////////////
// USEFUL //////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////
/**
 * Send an image message to the room.
 *
 * @param {Room} room - The room to send the image message to.
 * @param {object} messageData - Data of the image message.
 */
function sendImageMessage(room, messageData) {
    const imageFileName = messageData.imageName;
    const imagePath = path.join(imageFolder, imageFileName);
    // console.log('imagePath :>> ', imagePath);

    // Read the image from the file and convert it to base64
    fs.readFile(imagePath, 'base64', (err, imageBase64) => {
        if (err) {
            console.error('Error reading image file:', err);
        } else {
            const dataToSend = {
                username: messageData.from,
                image: imageBase64,
                roomID: messageData.roomID,
                time: messageData.time
            };
            console.log('sendImageMessage dataToSend :>> ', dataToSend);
            sendToRoom(room, 'new image message', dataToSend);
        }
    });
}

/**
 * Function that use CrytoJS to generate a symmetric key
 * @returns a symmetric key
 */
function generateSymmetricKey() {
    const symmetricKey = CryptoJS.lib.WordArray.random(256 / 8);
    return symmetricKey;
}

/**
 * Encrypts sensitive user data using a symmetric key and a public key.
 * @param {Object} data - The user data to be encrypted.
 *     - username {string} - The username.
 *     - email {string} - The email address.
 *     - hashedPassword {string} - The hashed password.
 * @param {string} publicKey - The public key used for encrypting the symmetric key.
 * @returns {Object} The encrypted user data.
 *     - username {string} - The username.
 *     - email {string} - The encrypted email address.
 *     - encryptedHashedPassword {string} - The encrypted hashed password.
 *     - encryptedSymmetricKey {string} - The encrypted symmetric key.
 */
function encryptConnectionData(data, publicKey) {
    const { username, email, hashedPassword } = data;
    const symmetricKeyEncrypt = generateSymmetricKey().toString();
    const encryptedHashedPassword = CryptoJS.AES.encrypt(hashedPassword, symmetricKeyEncrypt).toString();
    const encryptedEmail = CryptoJS.AES.encrypt(email, symmetricKeyEncrypt).toString();
    const encryptedSymmetricKey = cryptico.encrypt(symmetricKeyEncrypt, publicKey).cipher;

    const userDataRegistration = {
        username: username,
        email: encryptedEmail,
        encryptedHashedPassword: encryptedHashedPassword,
        encryptedSymmetricKey: encryptedSymmetricKey
    };

    return userDataRegistration;
}


/**
 * Decrypts encrypted user data using a private key and a decrypted symmetric key.
 * @param {Object} data - The encrypted user data to be decrypted.
 *     - username {string} - The username.
 *     - email {string} - The encrypted email address.
 *     - encryptedHashedPassword {string} - The encrypted hashed password.
 *     - encryptedSymmetricKey {string} - The encrypted symmetric key.
 * @param {string} privateKey - The private key used for decrypting the symmetric key.
 * @returns {Object} The decrypted user data.
 *     - username {string} - The username.
 *     - email {string} - The decrypted email address.
 *     - hashedPassword {string} - The decrypted hashed password.
 */
function decryptConnectionData(data, privateKey) {
    const { username, email, encryptedHashedPassword, encryptedSymmetricKey } = data;
    const decryptedSymmetricKey = cryptico.decrypt(encryptedSymmetricKey, privateKey).plaintext.toString();
    const decryptedHashedPassword = CryptoJS.AES.decrypt(encryptedHashedPassword, decryptedSymmetricKey).toString(CryptoJS.enc.Utf8);
    const decryptedEmail = CryptoJS.AES.decrypt(email, decryptedSymmetricKey).toString(CryptoJS.enc.Utf8);

    const userData = {
        username: username,
        email: decryptedEmail,
        hashedPassword: decryptedHashedPassword
    };

    return userData;
}

/**
 * Decrypts data using a symmetric key.
 * @param {string} data - The data to be decrypted.
 * @param {string} symmetricKey - The symmetric key used for decryption.
 * @returns {string} The decrypted data as a string.
 */
function decryptWithSymmetricKeyString(data, symmetricKey) {
    return CryptoJS.AES.decrypt(data, symmetricKey).toString(CryptoJS.enc.Utf8);
}


/**
 * Decrypts data using a public key.
 * @param {string} data - The data to be decrypted.
 * @param {string} publicKey - The public key used for decryption.
 * @returns {string} The decrypted data as a string.
 */
function decryptWithPublicKeyString(data, publicKey) {
    return cryptico.decrypt(data, publicKey).plaintext.toString();
}


/**
 * Retrieves the public key associated with a given username.
 * @param {string} username - The username to retrieve the public key for.
 * @returns {string|null} The public key if found, or null if the username is not found.
 */
function getPublicKeyByUsername(username) {
    const usersData = fs.readFileSync('persist/users.json', 'utf8');
    const users = JSON.parse(usersData);
    const user = users.find((user) => user.username === username);
    return user ? user.publicKey : null;
}


/**
 * Generates a server-side RSA key pair.
 * @returns {object} The generated RSA key pair.
 */
function generateServerKeyPair() {
    const passphrase = fs.readFileSync('passphrases/serverPassphrase.txt', 'utf8').trim();
    return cryptico.generateRSAKey(passphrase, 2048);
}


/**
 * Validates an email address.
 * @param {string} email - The email address to validate.
 * @returns {boolean} True if the email address is valid, false otherwise.
 */
function validateEmail(email) {
    const emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;

    if (!emailRegex.test(email) || email.length < 3 || email.length > 40)
        return false;

    return true;
}

/**
 * Validates a username input, ensuring it meets certain criteria.
 * @param {string} username - The username to validate.
 * @returns {string|boolean} The sanitized username if valid, false otherwise.
 */
function validateUsernameInput(username) {
    const alphanumericRegex = /^[a-zA-Z0-9]+$/;
    if (!alphanumericRegex.test(username) || username.length < 3 || username.length > 20) {
        return false;
    }
    const sanitizedInput = username.replace(/<[^>]+>/g, '');
    return sanitizedInput;
}

/**
 * Checks if an email address is already in use by any existing user.
 * @param {string} email - The email address to check.
 * @returns {boolean} True if the email is already in use, false otherwise.
 */
function emailAlreadyInUse(email) {
    const users = Users.getUsers();

    for (const userData of users) {
        if (userData.email === email) {
            return true;
        }
    }

    return false;
}


/**
 * Checks the identity of a user by matching their username, email, and hashed password.
 * @param {object} user - The user object containing the username, email, and hashed password to check.
        - name {string} - the username.
        - email {string} - the user email.
        - hashedPassword {string} - hashed password of the user
 * @returns {boolean} True if the user identity is valid, false otherwise.
 */
function checkUserIdentity(user) {
    const users = Users.getUsers();

    for (const userData of users) {
        if (userData.name === user.username) {
            if (userData.email === user.email) {
                if (userData.hashedPassword === user.hashedPassword) {
                    return true;
                }
            }
        }
    }

    return false;
}


///////////////////////////////
// Chatroom helper functions //
///////////////////////////////

/**
 * Sends an event with data to all connected sockets in a specific room.
 * @param {Room} room - The room object representing the target room.
 * @param {string} event - The name of the event to emit.
 * @param {*} data - The data to send along with the event.
 */
function sendToRoom(room, event, data) {
    io.to('room' + room.getId()).emit(event, data);
}

function persistImage(data) {
    let messagesData = [];
    try {
        const presentData = fs.readFileSync('persist/images-data.json', 'utf8');
        messagesData = JSON.parse(presentData);
    } catch (err) {
        console.error('Error reading image data:', err);
        return;
    }

    messagesData.push({
        imageName: data.imageName,
        from: data.from,
        roomID: data.roomID,
        time: data.time,
        direct: data.direct
    });

    try {
        fs.writeFileSync('persist/images-data.json', JSON.stringify(messagesData, null, 2));
        console.log('Image data persisted successfully.');
    } catch (err) {
        console.error('Error persisting image data:', err);
    }
}

/**
 * Persists a public message to the storage.
 * @param {Object} data - The data object representing the public message.
    * @param {string} data.username - The username of the message sender.
    * @param {string} data.message - The content of the message.
    * @param {string} data.time - The timestamp of the message.
    * @param {string} data.roomID - The ID of the room where the message was sent.
 */
function persistPublicMessage(data) {
    let messagesData = [];
    try {
        const presentData = fs.readFileSync('persist/public-messages.json', 'utf8');
        messagesData = JSON.parse(presentData);
    } catch (err) {
        console.error('Error reading message data:', err);
        return;
    }

    messagesData.push({
        username: data.username,
        message: data.message,
        time: data.time,
        roomID: data.roomID
    });

    try {
        fs.writeFileSync('persist/public-messages.json', JSON.stringify(messagesData, null, 2));
        console.log('Public message data persisted successfully.');
    } catch (err) {
        console.error('Error persisting public message data:', err);
    }
}

/**
 * Persists a private/direct message to the storage.
 * @param {Object} data - The data object representing the private/direct message.
 * @param {string} data.from - The username of the message sender.
 * @param {string} data.to - The username of the message recipient.
 * @param {string} data.roomID - The ID of the room where the message was sent.
 * @param {string} data.encryptedMessage - The encrypted content of the message.
 * @param {string} data.encryptedSymmetricKey - The encrypted symmetric key used for encryption.
 * @param {string} data.time - The timestamp of the message.
 * @param {boolean} data.direct - Indicates whether the message is a direct message.
 */
function persistPrivateDirectMessage(data) {
    let messagesData = [];
    try {
        const presentData = fs.readFileSync('persist/private-direct-messages.json', 'utf8');
        messagesData = JSON.parse(presentData);
    } catch (err) {
        console.error('Error reading message data:', err);
        return;
    }

    messagesData.push({
        from: data.from,
        to: data.to,
        roomID: data.roomID,
        encryptedMessage: data.encryptedMessage,
        encryptedSymmetricKey: data.encryptedSymmetricKey,
        time: data.time,
        direct: data.direct
    });

    try {
        fs.writeFileSync('persist/private-direct-messages.json', JSON.stringify(messagesData, null, 2));
        console.log('Private/Direct data persisted successfully.');
    } catch (err) {
        console.error('Error persisting private/direct message data:', err);
    }
}

/**
 * Persists a new user to the user data storage.
 * @param {Object} user - The user object representing the new user.
 * @param {string} user.name - The username of the new user.
 * @param {string} user.email - The email address of the new user.
 * @param {string} user.hashedPassword - The hashed password of the new user.
 * @param {string} user.publicKey - The public key of the new user.
 */
function persistNewUser(user) {
    // Read the existing user data from the JSON file
    let userData = [];
    try {
        const data = fs.readFileSync('persist/users.json', 'utf8');
        userData = JSON.parse(data);
    } catch (err) {
        // Handle file read error, if any
        console.error('Error reading user data:', err);
        return;
    }

    // Add the new user to the existing user data
    userData.push({
        username: user.name,
        email: user.email,
        hashedPassword: user.hashedPassword,
        publicKey: user.publicKey
    });

    // Write the updated user data back to the JSON file
    try {
        fs.writeFileSync('persist/users.json', JSON.stringify(userData, null, 2));
        console.log('User data persisted successfully.');
    } catch (err) {
        // Handle file write error, if any
        console.error('Error persisting user data:', err);
    }
}

/**
 * Loads rooms data from a JSON file and creates or updates rooms accordingly.
 */
function loadRoomsData() {
    try {
        const data = fs.readFileSync('persist/rooms.json', 'utf8');
        const roomData = JSON.parse(data);

        // Define the rooms to be added (by default)
        const newRooms = [
            { name: 'random', options: { forceMembership: true }, description: 'Random!' },
            { name: 'general', options: { forceMembership: true }, description: 'interesting things' },
            { name: 'private', options: { forceMembership: true, private: true }, description: 'some very private channel' }
        ];

        for (const roomObj of roomData) {
            const { id, name, description, options, members } = roomObj;
            console.log('roomObj:', roomObj);

            const room = Rooms.getRoomByName(name);

            if (!room) {
                const newRoom = Rooms.addRoom(name, options);
                newRoom.id = id;
                newRoom.description = description;

                for (const member of members) {
                    const { username, email, publicKey } = member;
                    const user = Users.getUser(username);
                    if (user) {
                        addUserToRoom(user, newRoom);
                    }
                }

                console.log('New room added:', newRoom);
            } else {
                console.log('Room already exists:', room);
            }
        }

        for (const newRoom of newRooms) {
            const room = Rooms.getRoomByName(newRoom.name);
            if (!room) {
                const createdRoom = Rooms.addRoom(newRoom.name, newRoom.options);
                createdRoom.description = newRoom.description;
                console.log('New room added:', createdRoom);
            } else {
                console.log('Room already exists:', room);
            }
        }

        console.log('Rooms data loaded successfully.');
    } catch (err) {
        console.error('Error loading rooms data:', err);
    }
}

/**
 * Loads private/direct messages data from a JSON file and sends them to their respective rooms.
 */
function loadPrivateDirectMessagesData() {
    try {
        const data = fs.readFileSync('persist/private-direct-messages.json', 'utf8');
        const messagesData = JSON.parse(data);

        for (const messageData of messagesData) {
            const room = Rooms.getRoom(messageData.roomID);
            if (room) {
                messageData.username = messageData.from;
                sendToRoom(room, 'new private-direct message', messageData);
                room.addMessage({
                    from: messageData.from,
                    to: messageData.to,
                    encryptedMessage: messageData.encryptedMessage,
                    encryptedSymmetricKey: messageData.encryptedSymmetricKey,
                    time: messageData.time
                });
            }
        }

        console.log('Private/Direct messages data loaded successfully');
    } catch (err) {
        console.log('Errpr loading private/direct messages data:', err);
    }
}

/**
 * Loads public messages data from a JSON file and sends them to their respective rooms.
 */
// function loadPublicMessagesData() {
//     try {
//         const data = fs.readFileSync('persist/public-messages.json', 'utf8');
//         const messagesData = JSON.parse(data);

//         for (const messageData of messagesData) {
//             const room = Rooms.getRoom(messageData.roomID);
//             if (room) {
//                 sendToRoom(room, 'new public message', messageData);
//                 room.addMessage({
//                     username: messageData.username,
//                     message: messageData.message,
//                     time: messageData.time
//                 });
//             }
//         }

//         console.log('Public messages data loaded successfully');
//     } catch (err) {
//         console.log('Error loading public messages data:', err);
//     }
// }

/**
 * Loads public and image messages data from JSON files and sends them to their respective rooms.
 */
function loadPublicMessagesData() {
    try {
        // Load public messages data
        const publicData = fs.readFileSync('persist/public-messages.json', 'utf8');
        const publicMessagesData = JSON.parse(publicData);

        // Load image messages data
        const imageData = fs.readFileSync('persist/images-data.json', 'utf8');
        const imagesData = JSON.parse(imageData);

        // Combine and sort all messages by time
        const allMessagesData = [...publicMessagesData, ...imagesData];
        allMessagesData.sort((a, b) => a.time - b.time);

        // Process and send messages to their respective rooms
        allMessagesData.forEach((messageData) => {
            const room = Rooms.getRoom(messageData.roomID);
            if (room) {
                if ('message' in messageData) {
                    sendToRoom(room, 'new public message', messageData);
                    room.addMessage({
                        username: messageData.username,
                        message: messageData.message,
                        time: messageData.time
                    });
                } else if ('imageName' in messageData) {
                    sendImageMessage(room, messageData);
                    room.addImage({
                        username: messageData.from,
                        image: messageData.imageName,
                        time: messageData.time
                    });
                }
            }
        });

        console.log('Messages data loaded successfully');
    } catch (err) {
        console.log('Error loading messages data:', err);
    }
}

/** Function to load users data from file
 */
function loadUsersData() {
    try {
        const data = fs.readFileSync('persist/users.json', 'utf8');
        const userData = JSON.parse(data);

        for (const userObj of userData)
            Users.addUser(userObj);

        console.log('Users data loaded successfully.');
    } catch (err) {
        console.error('Error loading users data:', err);
    }
}

/**
 * Persists room data by updating the 'persist/rooms.json' file with the provided room information.
 * Existing data for the same room ID is replaced.
 *
 * @param {Room} room - The room object containing the data to persist.
 */
function persistRoomData(room) {
    let roomData = [];
    try {
        const data = fs.readFileSync('persist/rooms.json', 'utf8');
        roomData = JSON.parse(data);
    } catch (err) {
        console.error('Error reading room data:', err);
        return;
    }

    roomData = roomData.filter(existingRoom => existingRoom.id !== room.getId());

    const roomObj = {
        id: room.getId(),
        name: room.name,
        description: room.description,
        options: {
            forceMembership: room.forceMembership,
            private: room.private,
            direct: room.direct
        },
        members: room.getMembers().map(member => {
            const userObj = Users.getUser(member.username);
            return {
                username: userObj.name,
                email: userObj.email,
                publicKey: userObj.publicKey
            };
        })
    };

    roomData.push(roomObj);

    try {
        fs.writeFileSync('persist/rooms.json', JSON.stringify(roomData, null, 2));
        console.log('Room data persisted successfully.');
    } catch (err) {
        console.error('Error persisting room data:', err);
    }
}

/**
 * Creates a new user with the provided data, adds the user to forced rooms,
 * persists the user data, and returns the created user.
 *
 * @param {Object} data - The data for the new user.
 * @returns {User} The created user.
 */
function newUser(data) {
    const user = Users.addUser(data);
    const rooms = Rooms.getForcedRooms();


    rooms.forEach(room => {
        addUserToRoom(user, room);
    });

    persistNewUser(user);

    return user;
}

/**
 * Creates a new room with the provided name, adds the given user to the room,
 * and returns the created room.
 *
 * @param {string} name - The name of the new room.
 * @param {User} user - The user to add to the room.
 * @param {Object} options - The options for the new room.
 * @returns {Room} The created room.
 */
function newRoom(name, user, options) {
    const room = Rooms.addRoom(name, options);
    addUserToRoom(user, room);

    return room;
}

/**
 * Creates a new channel (room) with the provided name, description, privacy,
 * and adds the given user to the room. Returns the created room.
 *
 * @param {string} name - The name of the new channel.
 * @param {string} description - The description of the new channel.
 * @param {boolean} private - Indicates if the channel is private.
 * @param {User} user - The user to add to the channel.
 * @returns {Room} The created channel (room).
 */
function newChannel(name, description, private, user) {
    return newRoom(name, user, {
        description: description,
        private: private
    });
}

/**
 * Creates a new direct room (private room) between two users, adds the users to the room,
 * and returns the created room.
 *
 * @param {User} user_a - The first user.
 * @param {User} user_b - The second user.
 * @returns {Room} The created direct room.
 */
function newDirectRoom(user_a, user_b) {
    const room = Rooms.addRoom(`Direct-${user_a.name}-${user_b.name}`, {
        direct: true,
        private: true,
    });

    addUserToRoom(user_a, room);
    addUserToRoom(user_b, room);

    return room;
}

/**
 * Retrieves the direct room between two users if it exists. If the direct room doesn't exist,
 * a new direct room is created and returned.
 *
 * @param {User} user_a - The first user.
 * @param {User} user_b - The second user.
 * @returns {Room} The direct room between the two users.
 */
function getDirectRoom(user_a, user_b) {
    const rooms = Rooms.getRooms().filter(r => r.direct
        && (
            (r.members[0].username == user_a.name && r.members[1].username == user_b.name) ||
            (r.members[1].username == user_a.name && r.members[0].username == user_b.name)
        ));

    if (rooms.length == 1)
        return rooms[0];
    else
        return newDirectRoom(user_a, user_b);
}

/**
 * Adds a user to a room by updating their subscriptions and room membership.
 * Sends a user update event to the room and persists the room data.
 *
 * @param {User} user - The user to add to the room.
 * @param {Room} room - The room to add the user to.
 */
function addUserToRoom(user, room) {
    user.addSubscription(room);
    room.addMember(user);

    sendToRoom(room, 'update_user', {
        room: room.getId(),
        username: user.name,
        action: 'added',
        members: room.getMembers()
    });


    persistRoomData(room);
}

/**
 * Removes a user from a room by updating their subscriptions and room membership.
 * Sends a user update event to the room and persists the room data.
 *
 * @param {User} user - The user to remove from the room.
 * @param {Room} room - The room to remove the user from.
 */
function removeUserFromRoom(user, room) {
    user.removeSubscription(room);
    room.removeMember(user);

    sendToRoom(room, 'update_user', {
        room: room.getId(),
        username: user.name,
        action: 'removed',
        members: room.getMembers()
    });

    persistRoomData(room);
}

/**
 * Sets the active state of a user and broadcasts the user state change to all connected sockets.
 *
 * @param {Socket} socket - The socket associated with the user.
 * @param {string} username - The username of the user.
 * @param {boolean} state - The active state of the user (true or false).
 */
function setUserActiveState(socket, username, state) {
    const user = Users.getUser(username);

    if (user)
        user.setActiveState(state);

    socket.broadcast.emit('user_state_change', {
        username: username,
        active: state
    });
}

/**
 * Checks if a user is a member of a specific room.
 *
 * @param {Room} room - The room to check.
 * @param {string} username - The username of the user.
 * @returns {boolean} True if the user is a member of the room, false otherwise.
 */
function isUserInRoom(room, username) {
    for (let member of room.members) {
        if (member.username === username) {
            return true;
        }
    }
    return false;
}

///////////////////////////
// IO connection handler //
///////////////////////////

const socketmap = {};

io.on('connection', (socket) => {
    let userLoggedIn = false;
    username = null;

    ///////////////////////
    // incomming message //
    ///////////////////////

    socket.on('new image message', (data) => {
        const time = new Date().getTime();
        const room = Rooms.getRoom(data.roomID);
        const uniqueFileName = `${data.roomID}_${time}.png`;
        const imagePath = `${imageFolder}/${uniqueFileName}`;

        // Save the image to the specified folder
        fs.writeFile(imagePath, data.image, 'base64', (err) => {
            if (err) {
                console.error('Error saving image:', err);
            } else {
                console.log('Image saved:', imagePath);
            }
        });
        const dataToPersist = {
            imageName: uniqueFileName,
            from: data.sender.username,
            roomID: data.roomID,
            time: time,
            direct: room.direct
        };
        const dataToSend = {
            username: data.sender.username,
            image: data.image,  // imageData is the base64 data URI of the image
            roomID: data.roomID,
            time: time
        };
        sendToRoom(room, 'new image message', dataToSend);
        persistImage(dataToPersist);
    });

    // Handles the event when a new public message is received
    socket.on('new public message', data => {
        if (userLoggedIn) {
            const user = decryptConnectionData(data.sender, serverKeyPair);

            // Verify user identity
            if (user && checkUserIdentity(user)) {
                const room = Rooms.getRoom(data.roomID);
                const userInRoom = isUserInRoom(room, user.username);
                const time = new Date().getTime();

                // Decrypt the symmetric key and the message
                const decryptedSymmetricKey = decryptWithPublicKeyString(data.encryptedSymmetricKey, serverKeyPair);
                const decryptedMessage = decryptWithSymmetricKeyString(data.encyptedMessage, decryptedSymmetricKey);

                // Encode username and decrypted message for security
                data.sender.username = he.encode(data.sender.username);
                const encodedDecryptedMessage = he.encode(decryptedMessage);
                if (room && userInRoom) {
                    const dataToSend = {
                        username: data.sender.username,
                        message: encodedDecryptedMessage,
                        time: time,
                        roomID: room.id
                    };
                    sendToRoom(room, 'new public message', dataToSend);
                    room.addMessage({
                        username: data.sender.username,
                        message: encodedDecryptedMessage,
                        time: time
                    });
                    persistPublicMessage(dataToSend); // Persist the public message data
                }
            } else
                socket.emit('wrong data provided');
        } else
            socket.emit('user logged out');
    });

    // Handles the event when a new private-direct message is received
    socket.on('new private-direct message', data => {
        if (userLoggedIn) {
            const user = decryptConnectionData(data.sender, serverKeyPair);
            // Verify user identity
            if (user && checkUserIdentity(user)) {
                const room = Rooms.getRoom(data.roomID);
                const userInRoom = isUserInRoom(room, user.username);
                const time = new Date().getTime();

                // Encode username and encrypted message for security
                data.sender.username = he.encode(data.sender.username);
                data.encryptedMessage = he.encode(data.encryptedMessage);   // Even if the user send a message without encrypting it first - we will encode it in case
                if (room && userInRoom) {
                    const dataToSend = {
                        from: data.sender.username,
                        to: data.recipient,
                        roomID: data.roomID,
                        encryptedMessage: data.encryptedMessage,
                        encryptedSymmetricKey: data.encryptedSymmetricKey,
                        time: time,
                        direct: room.direct
                    };
                    sendToRoom(room, 'new private-direct message', dataToSend);
                    room.addMessage({
                        from: data.sender.username,
                        to: data.recipient,
                        encryptedMessage: data.encryptedMessage,
                        encryptedSymmetricKey: data.encryptedSymmetricKey,
                        time: time
                    });
                    persistPrivateDirectMessage(dataToSend); // Persist the private-direct message data
                }
            } else
                socket.emit('wrong data provided');
        } else {
            socket.emit('user logged out');
        }
    });

    ////////////////////////////////////////////////////////////////
    // USEFUL //////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////

    // Handles the event when registering user data on the server in chunks
    socket.on('register_user_on_server_chunk', data => {
        const chunkData = data.chunkData;
        const currentChunk = data.currentChunk;
        const totalChunks = data.totalChunks;

        // Store the received chunk data in the userDataRegistrationChunks object
        userDataRegistrationChunks[currentChunk] = chunkData;

        // Check if all chunks have been received
        if (Object.keys(userDataRegistrationChunks).length === totalChunks) {
            let userDataRegistration = '';

            // Reconstruct the complete userDataRegistration from the chunks
            for (let i = 0; i < totalChunks; i++) {
                userDataRegistration += userDataRegistrationChunks[i];
            }

            // Parse the JSON data
            const completeData = JSON.parse(userDataRegistration);

            const newUserDataDecrypted = decryptConnectionData(completeData, serverKeyPair);
            newUserDataDecrypted.publicKey = completeData.userPublicKey;

            newUser(newUserDataDecrypted);

            // Clear the userDataRegistrationChunks object
            userDataRegistrationChunks = {};

            // Encrypt the user data and send back the registration success message
            const user = encryptConnectionData(newUserDataDecrypted, newUserDataDecrypted.publicKey);
            user.serverPublicKey = serverPublicKey;
            socket.emit('user_registered_successfully', user);
        }
    });

    // Handles the event when connecting user data on the server in chunks
    socket.on('validate_user_connection_chunk', data => {
        if (userLoggedIn)
            return;

        const chunkData = data.chunkData;
        const currentChunk = data.currentChunk;
        const totalChunks = data.totalChunks;

        userDataConnectionChunks[currentChunk] = chunkData;

        if (Object.keys(userDataConnectionChunks).length === totalChunks) {
            let userDataRegistration = '';

            // Reconstruct the complete userDataRegistration from the chunks
            for (let i = 0; i < totalChunks; i++) {
                userDataRegistration += userDataConnectionChunks[i];
            }

            // Parse the JSON data
            const completeData = JSON.parse(userDataRegistration);

            const userDataDecrypted = decryptConnectionData(completeData, serverKeyPair);
            userDataDecrypted.publicKey = completeData.userPublicKey;


            const user = encryptConnectionData(userDataDecrypted, getPublicKeyByUsername(userDataDecrypted.username));
            if (Users.getUser(userDataDecrypted.username) && checkUserIdentity(userDataDecrypted)) {
                user.serverPublicKey = serverPublicKey;
                socket.emit('user_identified_successfully', user);
            } else {
                socket.emit('wrong_data', user);
            }
            userDataConnectionChunks = {};
        }
    });

    // Handles the event to check if an email is available
    socket.on('check_email_available', data => {
        if (emailAlreadyInUse(data.email)) {
            socket.emit('email_unavailable', data);
        } else {
            socket.emit('email_available', data);
        }
    })

    // Handles the event to check if an email is validated/conform
    socket.on('check_email_validated', data => {
        if (validateEmail(data.email)) {
            data.serverPublicKey = serverPublicKey;
            socket.emit('email_authorized', data);
        } else {
            socket.emit('email_unauthorized', data);
        }
    })

    // Handles the event to check if a username is registered and validated/conform
    socket.on('check_username_registered_validated', username => {
        const username_sanitized = validateUsernameInput(username);
        if (!username_sanitized) {
            socket.emit('username_unauthorized', username);
            return;
        }

        if (Users.getUser(username) !== null)
            socket.emit('username_known', username);
        else
            socket.emit('username_unknown', username);
    });

    /////////////////////////////
    // request for direct room //
    /////////////////////////////


    socket.on('request_direct_room', req => {
        if (userLoggedIn) {
            const user_a = Users.getUser(req.to);
            const user_b = Users.getUser(req.me);

            if (user_a && user_b) {
                const room = getDirectRoom(user_a, user_b);
                const roomCID = 'room' + room.getId();
                socket.join(roomCID);
                if (socketmap[user_a.name])
                    socketmap[user_a.name].join(roomCID);

                socket.emit('update_direct_room', {
                    room: room,
                    moveto: true
                });
            }
        }
    });

    socket.on('add_channel', req => {
        if (userLoggedIn) {
            const userDecrypted = decryptConnectionData(req.user, serverKeyPair);
            if (userDecrypted && checkUserIdentity(userDecrypted)) {
                const user = Users.getUser(userDecrypted.username);
                const room = newChannel(req.name, req.description, req.private, user);
                const roomCID = 'room' + room.getId();
                socket.join(roomCID);

                socket.emit('update_room', {
                    room: room,
                    moveto: true
                });

                if (!room.private) {
                    const publicChannels = Rooms.getRooms().filter(r => !r.direct && !r.private);
                    socket.broadcast.emit('update_public_channels', {
                        publicChannels: publicChannels
                    });
                }
            }
        }
    });

    socket.on('join_channel', req => {
        if (userLoggedIn) {
            const user = Users.getUser(username);
            const room = Rooms.getRoom(req.id)

            if (!room.direct && !room.private) {
                addUserToRoom(user, room);

                const roomCID = 'room' + room.getId();
                socket.join(roomCID);

                socket.emit('update_room', {
                    room: room,
                    moveto: true
                });
            }
        }
    });


    socket.on('add_user_to_channel', req => {
        if (userLoggedIn) {
            const userDecrypted = decryptConnectionData(req.userData, serverKeyPair);
            if (userDecrypted && checkUserIdentity(userDecrypted)) {
                const user = Users.getUser(req.user);
                const room = Rooms.getRoom(req.channel)
                const userInRoom = isUserInRoom(room, userDecrypted.username);
                if (!room.direct && userInRoom) {
                    addUserToRoom(user, room);

                    if (socketmap[user.name]) {
                        const roomCID = 'room' + room.getId();
                        socketmap[user.name].join(roomCID);

                        socketmap[user.name].emit('update_room', {
                            room: room,
                            moveto: false
                        });
                    }
                }
            }
        }
    });

    socket.on('leave_channel', req => {
        if (userLoggedIn) {
            const userDecrypted = decryptConnectionData(req.user, serverKeyPair);
            if (userDecrypted && checkUserIdentity(userDecrypted)) {
                const user = Users.getUser(userDecrypted.username);
                const room = Rooms.getRoom(req.id);
                const userInRoom = isUserInRoom(room, userDecrypted.username);
                if (!room.direct && !room.forceMembership && userInRoom) {
                    removeUserFromRoom(user, room);

                    const roomCID = 'room' + room.getId();
                    socket.leave(roomCID);

                    socket.emit('remove_room', {
                        room: room.getId()
                    });
                }
            }
        }
    });

    ///////////////
    // user join //
    ///////////////

    // Handles the 'join' event when a user attempts to join the application
    socket.on('join', data => {
        if (userLoggedIn)
            return;

        socketmap[data.username] = socket;
        userLoggedIn = true;

        const user = Users.getUser(data.username);
        if (user && checkUserIdentity(data)) {
            // Join the user to subscribed rooms and retrieve public channels
            const rooms = user && user.getSubscriptions().map(s => {
                socket.join('room' + s);
                return Rooms.getRoom(s);
            });

            const publicChannels = Rooms.getRooms().filter(r => !r.direct && !r.private);

            // Emit 'login' event with user, room, and public channel data
            socket.emit('login', {
                users: Users.getUsers().map(u => ({ username: u.name, active: u.active })),
                rooms: rooms,
                publicChannels: publicChannels
            });

            // Set the user's active state to true
            setUserActiveState(socket, data.username, true);
        } else {
            // Emit 'wrong data provided' event if user data is invalid
            socket.emit('wrong data provided');
        }
    });

    ////////////////
    // reconnects //
    ////////////////

    socket.on('reconnect', () => {
        if (userLoggedIn) {
            setUserActiveState(socket, username, true);
        }
    });

    /////////////////
    // disconnects //
    /////////////////

    socket.on('disconnect', () => {
        if (userLoggedIn) {
            setUserActiveState(socket, username, false);
        }
    });

});
