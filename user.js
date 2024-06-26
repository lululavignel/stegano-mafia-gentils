let users = {};

class User {
    constructor(name, email, hashedPassword, publicKey, userType) {
        this.name = name;
        this.email = email;
        this.hashedPassword = hashedPassword; // Hash the password
        this.publicKey = publicKey;
        this.userType = userType;

        this.active = false;
        this.subscriptions = [];
    }

    getSubscriptions() {
        return this.subscriptions;
    }

    addSubscription(room) {
        const id = room.getId();

        if (this.subscriptions.indexOf(id) === -1)
            this.subscriptions.push(id);
    }

    removeSubscription(room) {
        const id = room.getId();

        const idx = this.subscriptions.indexOf(id);
        if (idx >= 0)
            this.subscriptions.splice(idx, 1);
    }

    setActiveState(b) {
        this.active = b;
    }

    getPublicKey() {
        return this.publicKey;
    }

    getIsMafia() {
        return this.isMafia;
    }

    getUserType() {
        return this.userType;
    }

}

module.exports = {
    addUser: (data) => {
        const { username, email, hashedPassword, publicKey, userType } = data;
        const user = new User(username, email, hashedPassword, publicKey, userType);
        users[username] = user;
        return user;
    },

    getUser: (name) => {
        for (const user of Object.values(users)) {
            if (user.name === name) {
                return user;
            }
        }
        return null;
    },

    getUsers: () => {
        return Object.values(users);
    },
}