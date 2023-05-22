var sqlite3 = require('sqlite3').verbose();

const dbfile = '../local/local.db';
let db = new sqlite3.Database(dbfile, (err) => {
    if (err) {
        console.log('Cannot open sqlite DB, error: ', err.message);
    }
    else {
        // Any init code here
    }
});

module.exports = db;
