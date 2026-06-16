const path = require('path');
const sqlite3 = require('sqlite3').verbose();

require('dotenv').config({ path: path.resolve(__dirname, '..', '.env') });
const dburl = process.env.DATABASE_URL || 'sqlite:local/local.db';
const dbfile = path.resolve(__dirname, '..', dburl.replace(/^sqlite:/, ''));

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
