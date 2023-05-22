const sqlite3 = require('sqlite3').verbose();
const dbfile = '../local/local.db';

const LoadDB = (callback) => {
    let db = new sqlite3.Database(dbfile, (err) => {
        if (err) {
            console.log('Cannot open sqlite DB, error: ', err.message);
        }
        else {
            callback(db);
        }
    });
};

module.exports = LoadDB;
