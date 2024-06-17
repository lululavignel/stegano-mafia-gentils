$(function () {
    // Initialize variables
    // TODO: try to add a new socket that will catch the loaded messages from the server and add them directly to the historic of the chats - soit pas con tu vas comprendre - raaah
    const $window = $(window);
    const $messages = $('.messages'); // Messages area
    const $inputMessage = $('#input-message'); // Input message input box
    const $usernameLabel = $('#user-name');
    const $roomList = $('#room-list');
    const $userList = $('#user-list');

    const username = getCookie('username');
    const hashedPassword = getCookie('hashedPassword');
    const email = getCookie('email');
    const userType = getCookie('userType');
    const serverPublicKey = getCookie('serverPublicKey');
    if (username && hashedPassword)
        userKeyPair = generateKeyPair({ username, hashedPassword });
    let connected = false;
    let socket = io();
    let modalShowing = false;

    window.user = { username: username, email: email, hashedPassword: hashedPassword };

    $usernameLabel.text(username);

    $('#addChannelModal').on('hidden.bs.modal', () => modalShowing = false)
        .on('show.bs.modal', () => modalShowing = true);

    ///////////////
    // User List //
    ///////////////

    let users = {};

    function updateUsers(p_users) {
        p_users.forEach(u => users[u.username] = u);
        updateUserList();
    }

    function updateUser(username, active, publicKey) {
        if (!users[username])
            users[username] = { username: username };


        users[username].active = active;
        updateUserList();
    }

    function updateUserList() {
        const $uta = $("#usersToAdd");
        $uta.empty();

        $userList.empty();
        for (let [un, usr] of Object.entries(users)) {
            if (username !== usr.username)
                $userList.append(`
          <li onclick="setDirectRoom(this)" data-direct="${usr.username}" class="${usr.active ? "online" : "offline"}">${usr.username}</li>
        `);
            // append it also to the add user list
            $uta.append(`
          <button type="button" class="list-group-item list-group-item-action" data-dismiss="modal" onclick="addToChannel('${usr.username}')">${usr.username}</button>
        `);
        };
    }

    ///////////////
    // Room List //
    ///////////////
    let rooms = [];

    function updateRooms(p_rooms) {
        rooms = p_rooms;

        updateRoomList();
    }

    function updateRoom(room) {
        rooms[room.id] = room;
        updateRoomList();
    }

    function removeRoom(id) {
        delete rooms[id];
        updateRoomList();
    }

    function updateRoomList() {
        $roomList.empty();
        rooms.forEach(r => {
            if (!r.direct)
                $roomList.append(`
            <li onclick="setRoom(${r.id})"  data-room="${r.id}" class="${r.private ? "private" : "public"}">${r.name}</li>
            `);
        });
    }


    function updateChannels(channels) {
        const c = $("#channelJoins");

        c.empty();
        channels.forEach(r => {
            if (!rooms[r.id])
                c.append(`
          <button type="button" class="list-group-item list-group-item-action" data-dismiss="modal" onclick="joinChannel(${r.id})">${r.name}</button>
        `);
        });
    }


    //////////////
    // Chatting //
    //////////////
    let currentRoom = false;

    /**
     * Function to set the current room and update the UI accordingly
     * @param {string|id} id - room id 
     */
    function setRoom(id) {
        let oldRoom = currentRoom;

        const room = rooms.find(room => {
            try {
                return room.id === id;
            } catch (error) {
                // Handle the error when room is undefined
                return false;
            }
        });
        currentRoom = room;


        $messages.empty();
        // Process and display messages for the room
        const messages = room.history.map((m) => {
            if (m.encryptedMessage && m.encryptedSymmetricKey) {
                const user = window.user;
                const { from, to, roomID, encryptedMessage, encryptedSymmetricKey, time, direct } = m;
                if (to === user.username) {
                    const decryptedSymmetricKey = decryptWithPublicKeyString(encryptedSymmetricKey, userKeyPair);
                    const decryptedMessage = decryptWithSymmetricKeyString(encryptedMessage, decryptedSymmetricKey);
                    const msg = {
                        username: from,
                        message: decryptedMessage,
                        time: time
                    };
                    addChatMessage(msg);
                    msg.room = roomID;
                    return msg;
                }
            } else if (m.image) {
                // console.log('setRoom m :>> ', m);
                addChatImage(m.username, m.image, m.time);
                const msg = { ...m };
                msg.room = m.roomID;
                return msg;
            } else {
                addChatMessage(m);
                const msg = { ...m };
                msg.room = room;
                return msg;
            }
        });

        $userList.find('li').removeClass("active");
        $roomList.find('li').removeClass("active");

        if (room.direct) {
            const idx = room.members.findIndex(member => member.username === window.user.username) == 0 ? 1 : 0;
            const user = room.members[idx];

            setDirectRoomHeader(user);

            $userList.find(`li[data-direct="${user}"]`)
                .addClass("active")
                .removeClass("unread")
                .attr('data-room', room.id);

        } else {
            $('#channel-name').text("#" + room.name);
            $('#channel-description').text(`👤 ${room.members.length} | ${room.description}`);
            $roomList.find(`li[data-room=${room.id}]`).addClass("active").removeClass("unread");
        }

        $('.roomAction').css('visibility', (room.direct || room.forceMembership) ? "hidden" : "visible");
    }
    window.setRoom = setRoom;

    function setDirectRoomHeader(user) {
        $('#channel-name').text(user.username);
        $('#channel-description').text(`Direct message with ${user.username}`);
    }

    function setToDirectRoom(user) {
        const me = window.user.username;
        setDirectRoomHeader(user);
        socket.emit('request_direct_room', { me: me, to: user });
    }

    window.setDirectRoom = (el) => {
        const user = el.getAttribute("data-direct");
        const room = el.getAttribute("data-room");

        if (room) {
            setRoom(parseInt(room));
        } else {
            setToDirectRoom(user);
        }
    }

    /**
     * Function to send a message
     */
    function sendMessage() {
        let message = $inputMessage.val();

        if (message && connected && currentRoom !== false) {
            $inputMessage.val('');
            const user = encryptConnectionDataWithoutHashing(window.user, serverPublicKey);
            const msgSymmetricKey = generateSymmetricKey().toString();

            // Check if the current room is private and/or direct
            if ((currentRoom.private !== false && currentRoom.direct !== true)
                || (currentRoom.private !== false && currentRoom.direct !== false)) {
                // Send the message to each member of the current room individually
                for (let i = 0; i < currentRoom.members.length; i++) {
                    const recipient = currentRoom.members[i];
                    const encodedMessage = he.encode(message);
                    const encryptedMessage = encryptWithSymmetricKeyString(encodedMessage, msgSymmetricKey);
                    const encryptedSymmetricKey = encryptWithPublicKeyString(msgSymmetricKey, recipient.userPublicKey);
                    const data = {
                        sender: user,
                        encryptedMessage: encryptedMessage,
                        encryptedSymmetricKey: encryptedSymmetricKey,
                        recipient: recipient.username,
                        roomID: currentRoom.id,
                        roomName: currentRoom.name
                    };
                    socket.emit('new private-direct message', data);
                }
            } else {
                // Send the message as a public message
                const encyptedMessage = encryptWithSymmetricKeyString(message, msgSymmetricKey);
                const encryptedSymmetricKey = encryptWithPublicKeyString(msgSymmetricKey, serverPublicKey);
                const msg = {
                    sender: user,
                    encyptedMessage: encyptedMessage,
                    encryptedSymmetricKey: encryptedSymmetricKey,
                    roomID: currentRoom.id
                };
                socket.emit('new public message', msg);
            }
        }
    }

    ////////////////////////////////////////////////////////////////
    // USEFUL //////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////

    $(document).on('click', '.run-prob-algorithm', function() {
        const username = $(this).data('username');
        const time = $(this).data('time');
        const imageSrc = $(this).data('image-src');
    
        // Envoyer les informations nécessaires au serveur
        const data = {
            user: username,
            date: time,
            source: imageSrc
        };
    
        socket.emit('run_probabilistic_algorithm', data);
    });

    document.getElementById('algorithm-choice').addEventListener('change', function() {
        const selectedValue = this.value;

        const matrixAlphaChoice = document.getElementById('alpha-matrix-container');
        const iteratorAlgorithm = document.getElementById('iterator-algorithm');
        if (selectedValue === "viterbi") {
            matrixAlphaChoice.style.display = 'block';
        } else {
            matrixAlphaChoice.style.display = 'none';
        }
    });

    $(document).on('click', '.reveal-hidden-message', function() {
        const username = $(this).data('username');
        const time = $(this).data('time');
        const imageSrc = $(this).data('image-src');
        const roomID = currentRoom.id;
        const data = { user: username, date: time, source: imageSrc, roomID: roomID };

        // imageFormPopup.style.display = 'block';
        // overlay.style.display = 'block';

        // // Use a timeout to ensure the element is ready to be focused
        // setTimeout(() => {
        //     secretMessageInput.focus();
        // }, 0);

        $('#passphrase-form-popup').data('revealData', data);
        passphraseFormPopup.style.display = 'block';
        overlay.style.display = 'block';
        passphraseAesInput.focus();
    });

    $('#passphrase-form').on('submit', function(e) {
        e.preventDefault();
        const passphrase = $('#aes-key-reveal').val();
        const data = $('#passphrase-form-popup').data('revealData');
        data.passphrase = passphrase;

        socket.emit('reveal_hidden_message', data);

        passphraseFormPopup.style.display = 'none';
        overlay.style.display = 'none';
        // $('#passphrase-form-popup').hide();
        // $('#overlay').hide();
        // $('#passphrase-form-popup')[0].reset();
        document.getElementById('passphrase-form').reset();
    });
  

    function addChatImage(username, imageSrc, timeObj) {
        let time = new Date(timeObj).toLocaleTimeString('en-US', {
            hour12: false,
            hour: "numeric",
            minute: "numeric"
        });
    
        let additionalButton = '';
        if (userType === 'steganaliste') {
            additionalButton = `
                <button class="run-prob-algorithm" data-username="${username}" data-time="${timeObj}" data-image-src="${imageSrc}">
                    Lancer algorithme probabiliste
                </button>
            `;
        } else if (userType === 'mafia') {
            additionalButton = `
                <button class="reveal-hidden-message" data-username="${username}" data-time="${timeObj}" data-image-src="${imageSrc}">
                    Retrouver le message caché
                </button>
            `;
        }
    
        $messages.append(`
            <div class="message">
                <div class="message-avatar"></div>
                <div class="message-textual">
                    <span class="message-user">${username}</span>
                    <span class="message-time">${time}</span>
                    <img src="${imageSrc}" alt="Image" class="message-image">
                    ${additionalButton}
                </div>
            </div>
        `);
    
        $messages[0].scrollTop = $messages[0].scrollHeight;
    }

    function addChatMessage(msg) {
        let time = new Date(msg.time).toLocaleTimeString('en-US', {
            hour12: false,
            hour: "numeric",
            minute: "numeric"
        });

        $messages.append(`
      <div class="message">
        <div class="message-avatar"></div>
        <div class="message-textual">
          <span class="message-user">${msg.username}</span>
          <span class="message-time">${time}</span>
          <span class="message-content">${msg.message}</span>
        </div>
      </div>
    `);

        $messages[0].scrollTop = $messages[0].scrollHeight;
    }

    function messageNotify(msg) {
        if (msg.direct)
            $userList.find(`li[data-direct="${msg.username}"]`).addClass('unread');
        else
            $roomList.find(`li[data-room=${msg.room}]`).addClass("unread");
    }


    function addChannel() {
        const name = $("#inp-channel-name").val();
        const description = $("#inp-channel-description").val();
        const private = $('#inp-private').is(':checked');

        const user = encryptConnectionDataWithoutHashing(window.user, serverPublicKey);

        socket.emit('add_channel', { user: user, name: name, description: description, private: private });
    }
    window.addChannel = addChannel;


    function joinChannel(id) {
        socket.emit('join_channel', { id: id });
    }
    window.joinChannel = joinChannel;

    function addToChannel(user) {
        const userData = encryptConnectionDataWithoutHashing(window.user, serverPublicKey);
        socket.emit('add_user_to_channel', { userData: userData, channel: currentRoom.id, user: user });
    }
    window.addToChannel = addToChannel;

    function leaveChannel() {
        const user = encryptConnectionDataWithoutHashing(window.user, serverPublicKey);
        socket.emit('leave_channel', { user: user, id: currentRoom.id });
    }
    window.leaveChannel = leaveChannel;

    // Get a reference to the drop zone element
    const dropZone = document.getElementById('drop-zone');
    const imageFormPopup = document.getElementById('image-form-popup');
    const passphraseFormPopup = document.getElementById('passphrase-form-popup');
    const algorithmFormPopup = document.getElementById('algorithm-form-popup');
    const overlay = document.getElementById('overlay');
    const secretMessageInput = document.getElementById('message-secret');
    const passphraseAesInput = document.getElementById('aes-key-reveal');
    let droppedImageFile = null;

    // Prevent the default behavior of the browser when a file is dropped
    dropZone.addEventListener('dragover', (e) => {
        e.preventDefault();
        dropZone.classList.add('drag-over');
    });

    dropZone.addEventListener('dragleave', () => {
        dropZone.classList.remove('drag-over');
    });

    dropZone.addEventListener('drop', (e) => {
        e.preventDefault();
        dropZone.classList.remove('drag-over');

        const files = e.dataTransfer.files;

        if (files.length > 0) {
            // Assume the first file is the image
            droppedImageFile = files[0];

            if (userType === 'mafia') {
                // Show the form popup and overlay
                imageFormPopup.style.display = 'block';
                overlay.style.display = 'block';
    
                // Use a timeout to ensure the element is ready to be focused
                setTimeout(() => {
                    secretMessageInput.focus();
                }, 0);
            } else {
                handleImageFile(droppedImageFile)
            }

        }
    });

    // Add a click event to open the file dialog for manual file selection
    dropZone.addEventListener('click', () => {
        const fileInput = document.createElement('input');
        fileInput.type = 'file';
        fileInput.accept = 'image/*';
        fileInput.style.display = 'none';
        document.body.appendChild(fileInput);

        fileInput.addEventListener('change', (e) => {
            const files = e.target.files;

            if (files.length > 0) {
                droppedImageFile = files[0];

                if (userType === 'steganaliste') {
                    handleImageFile(droppedImageFile)
                } else {
                    imageFormPopup.style.display = 'block';
                    overlay.style.display = 'block';
        
                    setTimeout(() => {
                        secretMessageInput.focus();
                    }, 0);
                }
            }
        });

        fileInput.click();
    });

    $(document).on('click', '.run-prob-algorithm', function() {
        const username = $(this).data('username');
        const time = $(this).data('time');
        const imageSrc = $(this).data('image-src');
        const roomID = currentRoom.id;
        const data = { user: username, date: time, source: imageSrc, roomID: roomID };

        $('#algorithm-form-popup').data('probData', data);
        algorithmFormPopup.style.display = 'block';
        overlay.style.display = 'block';
        $('#prob-algorithm-choice').focus();
    });

    $('#algorithm-form').on('submit', function(e) {
        e.preventDefault();
        const selectedAlgorithm = $('#prob-algorithm-choice').val();
        const data = $('#algorithm-form-popup').data('probData');
        data.algorithm = selectedAlgorithm;

        socket.emit('run_probabilistic_algorithm', data);

        algorithmFormPopup.style.display = 'none';
        overlay.style.display = 'none';
        $('#algorithm-form')[0].reset();
    });

    document.getElementById('image-form').addEventListener('submit', (e) => {
        e.preventDefault();
    
        const secretMessage = document.getElementById('message-secret').value;
        const dropdownChoice = document.getElementById('algorithm-choice').value;
        const imagePercentage = document.getElementById('image-percentage').value;
        const keyFile = document.getElementById('key-file').value;
        const iteratorAlgorithm = document.getElementById('iterator-algorithm').value;
        const alphaMatrix = document.getElementById('alpha-matrix') ? document.getElementById('alpha-matrix').value : null;
    
        handleImageFile(droppedImageFile, secretMessage, dropdownChoice, imagePercentage, keyFile, iteratorAlgorithm, alphaMatrix);
    
        imageFormPopup.style.display = 'none';
        overlay.style.display = 'none';
    
        document.getElementById('image-form').reset();
    
        document.getElementById('alpha-matrix-container').style.display = 'none';
    });
    

    document.getElementById('passphrase-form').addEventListener('submit', (e) => {
        e.preventDefault();

        const aesKey = document.getElementById('aes-key-reveal').value;

    });
    
    function handleImageFile(imageFile, secretMessage, dropdownChoice, imagePercentage, keyFile, iteratorAlgorithm, alphaMatrix) {
        if (imageFile.type.startsWith('image/')) {
            const reader = new FileReader();
    
            reader.onload = (event) => {
                const imageData = event.target.result;
                const data = {
                    image: imageData,
                    secretMessage: secretMessage ? secretMessage : "",
                    dropdownChoice: dropdownChoice ? dropdownChoice : "",
                    imagePercentage: imagePercentage ? imagePercentage : "",
                    keyFile: keyFile ? keyFile : "",
                    iteratorAlgorithm: iteratorAlgorithm ? iteratorAlgorithm : "",
                    alphaMatrix: alphaMatrix ? alphaMatrix : ""
                };
                sendImage(data);
            };
    
            reader.readAsDataURL(imageFile);
        } else {
            alert('Please select a valid image file.');
        }
    }
    
    function sendImage(data) {
        if (connected && currentRoom !== false) {
            const user = encryptConnectionDataWithoutHashing(window.user, serverPublicKey);
            const msgSymmetricKey = generateSymmetricKey().toString();
            
            const msg = {
                sender: user,
                image: data.image,
                secretMessage: data.secretMessage,
                stegaChoice: data.dropdownChoice,
                imagePercentage: data.imagePercentage,
                keyFile: data.keyFile,
                iteratorAlgorithm: data.iteratorAlgorithm,
                alphaMatrix: data.alphaMatrix,
                roomID: currentRoom.id,
            };

            console.log("sendImage msg", msg);

            socket.emit('new image message', msg);
        }
    }

    /////////////////////
    // Keyboard events //
    /////////////////////

    $window.keydown(event => {
        // Check if the modalShowing variable is true or if the form popup is visible
        if (modalShowing || imageFormPopup.style.display === 'block' || passphraseFormPopup.style.display === 'block') {
            return;
        }
    
        // Autofocus the current input when a key is typed
        if (!(event.ctrlKey || event.metaKey || event.altKey)) {
            $inputMessage.focus();
        }
    
        // When the client hits ENTER on their keyboard
        if (event.which === 13) {
            sendMessage();
            // Prevent default to avoid adding new lines
            event.preventDefault();
        }
    });
    



    ///////////////////
    // server events //
    ///////////////////

    // Whenever the server emits -login-, log the login message
    socket.on('login', (data) => {
        connected = true;
        updateUsers(data.users);
        updateRooms(data.rooms);
        updateChannels(data.publicChannels);

        if (data.rooms.length > 0) {
            setRoom(data.rooms[0].id);
        }
    });

    socket.on('update_public_channels', (data) => {
        updateChannels(data.publicChannels);
    });

    // Handles the event when a new private-direct message is received
    socket.on('new private-direct message', data => {
        let message;
        const user = window.user;
        data.from = he.decode(data.from);
        const { from, to, roomID, encryptedMessage, encryptedSymmetricKey, time, direct } = data;
        if (to === user.username) {
            const room = rooms.find(room => {
                try {
                    return room.id === roomID;
                } catch (error) {
                    return false;
                }
            });
            if (room) {
                const decryptedSymmetricKey = decryptWithPublicKeyString(encryptedSymmetricKey, userKeyPair);
                const decryptedMessage = decryptWithSymmetricKeyString(encryptedMessage, decryptedSymmetricKey);
                message = {
                    username: from,
                    message: decryptedMessage,
                    time: time
                };
                room.history.push(message);
            }

            if (currentRoom.id === room.id) {
                addChatMessage(message);
            } else {
                messageNotify(message);
            }
        }
    });

    // Handles the event when a new public message is received
    socket.on('new public message', data => {
        const roomId = data.roomID;
        data.username = he.decode(data.username);
        // data.message = he.decode(data.message);
        const room = rooms.find(room => {
            try {
                return room.id === roomId;
            } catch (error) {
                return false;
            }
        });
        if (room) {
            room.history.push(data);
        }

        if (roomId == currentRoom.id)
            addChatMessage(data);
        else
            messageNotify(data);
    });

    socket.on('new image message', data => {
        const username = data.username;
        const imageSrc = data.image;
        const time = data.time;
        const roomId = data.roomID;

        const room = rooms.find(room => {
            try {
                return room.id === roomId;
            } catch (error) {
                return false;
            }
        });
        console.log('room new image message :>> ', room);
        if (room) {
            console.log("image pushed in history");
            room.history.push(data);
        }


        if (roomId == currentRoom.id)
            addChatImage(username, imageSrc, time);

    });

    socket.on('hidden_message_revealed', (data) => {
        if (userType === 'mafia') {
            const roomId = data.roomID;
            const secretMessage = data.secretMessage;

            const room = rooms.find(room => {
                try {
                    return room.id === roomId;
                } catch (error) {
                    return false;
                }
            });

            if (room) {
                const msg = {
                    username: 'Server',
                    message: secretMessage,
                    time: new Date().getTime(),
                    room: roomId
                };

                room.history.push(msg);

                if (roomId === currentRoom.id) {
                    addChatMessage(msg);
                } else {
                    messageNotify(msg);
                }
            }
        }
    });

    socket.on('update_user', data => {
        const room = rooms[data.room];
        if (room) {
            room.members = data.members;

            if (room === currentRoom)
                setRoom(data.room);
        }
    });

    socket.on('user_state_change', (data) => {
        updateUser(data.username, data.active, data.publicKey);
    });

    socket.on('update_room', data => {
        updateRoom(data.room);
        if (data.moveto)
            setRoom(data.room.id);
        location.reload();
    });

    socket.on('update_direct_room', data => {
        updateRoom(data.room);
        if (data.moveto)
            setRoom(data.room.id);
    });

    socket.on('remove_room', data => {
        removeRoom(data.room);
        if (currentRoom.id == data.room)
            setRoom(0);
        location.reload();
    });

    socket.on('wrong data provided', () => {
        window.location.href = '/unknown.html';
    });


    ////////////////
    // Connection //
    ////////////////

    socket.on('connect', () => {
        socket.emit('join', user);
    });

    socket.on('disconnect', () => { });

    socket.on('reconnect', () => {
        socket.emit('join', user);
    });

    socket.on('reconnect_error', () => { });
});
