const path = require('path');
require('dotenv').config({ path: path.resolve(__dirname, '..', '.env') });

const express = require('express');
const app = express();
// Setup rendering engine
app.engine('html', require('ejs').renderFile);
app.set('view engine', 'html');

// Setup paths for Bootstrap
app.use(express.static(__dirname + '\\node_modules\\bootstrap\\dist'));
app.use(express.static('public'));

// Setup needed to handle POST requests
app.use(express.json());
app.use(express.urlencoded({ extended: false }));

const LoadDB = require('./database');
const e_prefix = process.env.EMOJI_PREFIX || '';

app.get('/:token', async (req, res) => {
    const token = req.params['token'];

    LoadDB((db) => {
        const sql = `
            SELECT t.season, t.week,
                   u.avatar, po.name, po.favteam,
                   ft.match AS feat_id, ft.target AS feat_val
            FROM pick_tokens AS t
                JOIN poolers AS po ON po.id = t.poolerid
                JOIN users   AS u  ON u.id  = po.userid
                LEFT JOIN features AS ft ON ft.season = t.season AND ft.week = t.week
            WHERE t.token = ?
        `;
        db.get(sql, token, async (err, row) => {
            // Bad/expired token
            if (err || !row) {
                if (err) {
                    console.log('Could not query DB for token, err: ', err.message);
                }
                res.render('error.html');
                return;
            }

            const avatar = row['avatar'];
            const username = row['name'];
            const favteam = row['favteam'];
            const season = row['season'];
            const week = row['week'];

            const feat_id = row['feat_id'];
            const feat_val = row['feat_val'];

            var w = week;
            var stype = 2;
            if (week == 19 || week == '19') { w = 1; stype = 3; }
            else if (week == 20 || week == '20') { w = 2; stype = 3; }
            else if (week == 21 || week == '21') { w = 3; stype = 3; }
            else if (week == 22 || week == '22') { w = 5; stype = 3; }

            const partial_url = "https://site.api.espn.com/apis/site/v2/sports/football/nfl/scoreboard";
            const url = `${partial_url}?dates=${season}&seasontype=${stype}&week=${w}`;
            const result = await fetch(url);
            const json = await result.json();

            let matches = [];
            let forcedid = 0;
            if (json['events']) {
                matches = json['events'].map((m) => {
                    const match = {};
                    match['idEvent'] = m['id'];
                    match['date'] = new Date(m['date']);

                    const teams = m['competitions'][0]['competitors'];
                    const hteam = teams[0];
                    const ateam = teams[1];


                    match['homeTeam'] = hteam['team']['abbreviation'];
                    match['awayTeam'] = ateam['team']['abbreviation'];
                    match['strHomeTeam'] = hteam['team']['displayName'];
                    match['strAwayTeam'] = ateam['team']['displayName'];

                    match['homeRecordAll'] = (hteam['records']) ? hteam['records'][0] : 0;
                    match['homeRecordAlt'] = (hteam['records']) ? hteam['records'][1] : 0;
                    match['awayRecordAll'] = (ateam['records']) ? ateam['records'][0] : 0;
                    match['awayRecordAlt'] = (ateam['records']) ? ateam['records'][2] : 0;

                    if (m['id'] == feat_id) {
                        match['featured'] = true;
                    }

                    if (match['awayTeam'] === favteam || match['homeTeam'] === favteam) {
                        forcedid = m['id'];
                    }
                    return match;
                });
            }

            res.render('picks.html', {
                season, week,
                token,
                username, favteam, avatar,
                matches, forcedid,
                feat_val, e_prefix
            });
        });
    });
});

app.post('/submit/:token', (req, res) => {
    const token = req.params['token'];
    const { matchids, favteam, forcedid, feat_pick, ...picks } = req.body;

    if (forcedid && forcedid !== '0') {
        picks[forcedid] = favteam;
    }

    const entries = Object.entries(picks);
    if (entries.length === 0) {
        console.log('Empty pick submission, refusing to insert');
        res.render('error.html');
        return;
    }

    LoadDB((db) => {
        db.get('SELECT poolerid, season, week FROM pick_tokens WHERE token = ?', token, (err, row) => {
            if (err || !row) {
                console.log(err);
                res.render('error.html');
                return;
            }
            const { poolerid, season, week } = row;

            db.run('BEGIN', (err) => {
                if (err) {
                    console.log(err);
                    res.render('error.html');
                    return;
                }

                const parentInsert = `
                    INSERT INTO picks (season, week, poolerid, featurepick)
                    VALUES (?, ?, ?, ?)
                `;
                db.run(parentInsert, [season, week, poolerid, Number(feat_pick)], function (err) {
                    if (err) {
                        db.run('ROLLBACK');
                        console.log(err);
                        res.render('error.html');
                        return;
                    }

                    const pickid = this.lastID;
                    const placeholders = entries.map(() => '(?, ?, ?)').join(', ');
                    const params = entries.flatMap(([matchid, team]) => [pickid, matchid, team]);

                    const childInsert = `
                        INSERT INTO match_picks (pickid, matchid, team)
                        VALUES ${placeholders}
                    `;
                    db.run(childInsert, params, (err) => {
                        if (err) {
                            db.run('ROLLBACK');
                            console.log(err);
                            res.render('error.html');
                            return;
                        }

                        db.run('DELETE FROM pick_tokens WHERE token = ?', token, () => {
                            db.run('COMMIT', () => res.render('success.html'));
                        });
                    });
                });
            });
        });
    });
});

app.get('/capsule/:token', (req, res) => {
    const token = req.params['token'];

    LoadDB((db) => {
        db.get('SELECT season FROM pick_tokens WHERE token = ?', token, (err, row) => {
            if (err || !row) {        // bad/expired token
                if (err) console.log(err);
                res.render('error.html');
                return;
            }

            res.render('playoffs.html', {
                afcTeams,
                nfcTeams,
                token,
                season: row['season'],
                e_prefix
            });
        });
    });
});

const CAP_TYPE = { DIVISION_WIN: 0, WILDCARD: 1 };
const CAP_CONF = { NFC: 0, AFC: 1 };
const DIV_ORDER = ['North', 'South', 'East', 'West']; // index = division column value
const TEAM_IDX = 6; // position of the team short name in a capsule row

app.post('/capsule-submit/:token', (req, res) => {
    const token = req.params['token'];
    const { afcWinners, nfcWinners, afcWildcards, nfcWildcards } = req.body;

    // Parse the JSON strings from the form (pick payload only)
    const afcWinnersObj = JSON.parse(afcWinners);
    const nfcWinnersObj = JSON.parse(nfcWinners);
    const afcWildcardsArr = JSON.parse(afcWildcards);
    const nfcWildcardsArr = JSON.parse(nfcWildcards);

    LoadDB((db) => {
        // Identity (poolerid, season) comes from the token, never the body
        db.get('SELECT poolerid, season FROM pick_tokens WHERE token = ?', token, (err, row) => {
            if (err || !row) {
                if (err) console.log(err);
                res.render('error.html');
                return;
            }

            const { poolerid, season } = row;

            const rows = [];
            DIV_ORDER.forEach((div, divIdx) => {
                rows.push([season, poolerid, CAP_TYPE.DIVISION_WIN, CAP_CONF.AFC, divIdx, 0, GetTeamShortName(afcWinnersObj[div])]);
                rows.push([season, poolerid, CAP_TYPE.DIVISION_WIN, CAP_CONF.NFC, divIdx, 0, GetTeamShortName(nfcWinnersObj[div])]);
            });
            afcWildcardsArr.forEach((name, slot) => {
                rows.push([season, poolerid, CAP_TYPE.WILDCARD, CAP_CONF.AFC, 0, slot, GetTeamShortName(name)]);
            });
            nfcWildcardsArr.forEach((name, slot) => {
                rows.push([season, poolerid, CAP_TYPE.WILDCARD, CAP_CONF.NFC, 0, slot, GetTeamShortName(name)]);
            });

            if (rows.length !== 14 || rows.some(r => !r[TEAM_IDX])) {
                console.log('Incomplete capsule submission, refusing to insert');
                res.render('error.html');
                return;
            }

            const placeholders = rows.map(() => '(?, ?, ?, ?, ?, ?, ?)').join(', ');
            const insert = `
                INSERT INTO capsules (season, poolerid, type, conference, division, slot, team)
                VALUES ${placeholders}
            `;
            db.run(insert, rows.flat(), (err) => {
                if (err) {
                    console.log(err);
                    res.render('error.html');
                    return;
                }
                // Consume the token so the link can't be replayed or double-submitted
                db.run('DELETE FROM pick_tokens WHERE token = ?', token, () => res.render('success.html'));
            });
        });
    });
});

app.get('/capsule-repicks/:token', (req, res) => {
    const token = req.params['token'];

    LoadDB((db) => {
        const sql = `
            SELECT t.season, p.repicks, c.type, c.conference, c.division, c.slot, c.team
            FROM pick_tokens AS t
            JOIN poolers AS p
                ON p.id = t.poolerid
            JOIN capsules AS c
                ON c.poolerid = t.poolerid AND c.season = t.season
            WHERE token = ?
        `;
        db.all(sql, token, (err, rows) => {
            if (err || !rows || rows.length === 0) {
                if (err) console.log(err);
                res.render('error.html');
                return;
            }

            const afcWinners = rows
                .filter(e => e.type === 0 && e.conference === 1)
                .reduce((acc, e) => {
                    acc[DIV_ORDER[e.division]] = e.team;
                    return acc;
                }, {});

            const nfcWinners = rows
                .filter(e => e.type === 0 && e.conference === 0)
                .reduce((acc, e) => {
                    acc[DIV_ORDER[e.division]] = e.team;
                    return acc;
                }, {});

            let afcWildcards = rows.filter((e) => {
                return e.type === 1 && e.conference === 1;
            });
            afcWildcards.sort((a, b) => a.slot - b.slot);
            afcWildcards = afcWildcards.map((e) => e.team);

            let nfcWildcards = rows.filter((e) => {
                return e.type === 1 && e.conference === 0;
            });
            nfcWildcards.sort((a, b) => a.slot - b.slot);
            nfcWildcards = nfcWildcards.map((e) => e.team);

            res.render('playoff-repicks.html', {
                afcWinners,
                nfcWinners,
                afcWildcards,
                nfcWildcards,
                repicks: rows[0]['repicks'],
                afcTeams, nfcTeams,
                token, season: rows[0]['season'],
                e_prefix
            });
        });
    });
});

app.post('/capsule-repicks-submit/:token', (req, res) => {
    const token = req.params['token'];
    const { afcWinners, nfcWinners, afcWildcards, nfcWildcards } = req.body;

    // Parse the JSON strings from the form (pick payload only)
    const afcWinnersObj = JSON.parse(afcWinners);
    const nfcWinnersObj = JSON.parse(nfcWinners);
    const afcWildcardsArr = JSON.parse(afcWildcards);
    const nfcWildcardsArr = JSON.parse(nfcWildcards);

    LoadDB((db) => {
        const sql = `
            SELECT pt.poolerid, pt.season, pl.repicks
            FROM pick_tokens AS pt
            JOIN poolers AS pl
                ON pt.poolerid = pl.id
            WHERE token = ?
        `;
        db.get(sql, token, (err, row) => {
            if (err || !row) {
                if (err) console.log(err);
                res.render('error.html');
                return;
            }

            const { poolerid, season, repicks } = row;

            db.all('SELECT id, type, conference, division, slot, team FROM capsules WHERE season = ? AND poolerid = ?', season, poolerid, (err, rows) => {
                if (err || !rows) { if (err) console.log(err); res.render('error.html'); return; }

                const changes = {};
                for (const r of rows) {
                    const is_win = r['type'] === 0;
                    const is_afc = r['conference'] === 1;

                    let new_team = '';
                    if (is_win) {
                        new_team = (is_afc) ? afcWinnersObj[DIV_ORDER[r['division']]] : nfcWinnersObj[DIV_ORDER[r['division']]];
                    }
                    else {
                        new_team = (is_afc) ? afcWildcardsArr[r['slot']] : nfcWildcardsArr[r['slot']];
                    }

                    if (!new_team) {
                        console.log('Incomplete re-pick submission, refusing to apply');
                        res.render('error.html');
                        return;
                    }

                    if (r['team'] !== new_team) {
                        changes[r['id']] = new_team;
                    }
                }

                const ids = Object.keys(changes);
                if (ids.length > repicks) {
                    console.log("Trying to change more than {repicks} capsule picks");
                    res.render('error.html');
                    return;
                }
                if (ids.length <= 0) {
                    res.render('success.html');
                    return;
                }

                db.serialize(() => {
                    db.run('BEGIN');

                    const stmt = db.prepare('UPDATE capsules SET team = ? WHERE id = ?');
                    for (const id of ids) {
                        stmt.run(changes[id], id);
                    }
                    stmt.finalize();

                    db.run('UPDATE poolers SET repicks = repicks - ? WHERE id = ?', ids.length, poolerid);
                    db.run('DELETE FROM pick_tokens WHERE token = ?', token);

                    db.run('COMMIT', (err) => {
                        if (err) {
                            console.log(err);
                            res.render('error.html');
                            return;
                        }
                        res.render('success.html');
                    });
                });
            });
        });
    });
});


const port = 3000;

app.listen(port, () => {
    console.log(`Picks page application, listening on port ${port}`);
});

const afcTeams = [
  { sname: 'BAL', name: 'Baltimore Ravens', division: 'North' },
  { sname: 'CIN', name: 'Cincinnati Bengals', division: 'North' },
  { sname: 'CLE', name: 'Cleveland Browns', division: 'North' },
  { sname: 'PIT', name: 'Pittsburgh Steelers', division: 'North' },
  { sname: 'HOU', name: 'Houston Texans', division: 'South' },
  { sname: 'IND', name: 'Indianapolis Colts', division: 'South' },
  { sname: 'JAX', name: 'Jacksonville Jaguars', division: 'South' },
  { sname: 'TEN', name: 'Tennessee Titans', division: 'South' },
  { sname: 'BUF', name: 'Buffalo Bills', division: 'East' },
  { sname: 'MIA', name: 'Miami Dolphins', division: 'East' },
  { sname: 'NYJ', name: 'New York Jets', division: 'East' },
  { sname: 'NE', name: 'New England Patriots', division: 'East' },
  { sname: 'KC', name: 'Kansas City Chiefs', division: 'West' },
  { sname: 'LV', name: 'Las Vegas Raiders', division: 'West' },
  { sname: 'LAC', name: 'Los Angeles Chargers', division: 'West' },
  { sname: 'DEN', name: 'Denver Broncos', division: 'West' }
];

const nfcTeams = [
  { sname: 'DET', name: 'Detroit Lions', division: 'North' },
  { sname: 'GB', name: 'Green Bay Packers', division: 'North' },
  { sname: 'MIN', name: 'Minnesota Vikings', division: 'North' },
  { sname: 'CHI', name: 'Chicago Bears', division: 'North' },
  { sname: 'TB', name: 'Tampa Bay Buccaneers', division: 'South' },
  { sname: 'ATL', name: 'Atlanta Falcons', division: 'South' },
  { sname: 'NO', name: 'New Orleans Saints', division: 'South' },
  { sname: 'CAR', name: 'Carolina Panthers', division: 'South' },
  { sname: 'DAL', name: 'Dallas Cowboys', division: 'East' },
  { sname: 'PHI', name: 'Philadelphia Eagles', division: 'East' },
  { sname: 'NYG', name: 'New York Giants', division: 'East' },
  { sname: 'WSH', name: 'Washington Commanders', division: 'East' },
  { sname: 'SF', name: 'San Francisco 49ers', division: 'West' },
  { sname: 'LAR', name: 'Los Angeles Rams', division: 'West' },
  { sname: 'SEA', name: 'Seattle Seahawks', division: 'West' },
  { sname: 'ARI', name: 'Arizona Cardinals', division: 'West' }
];

const lNameMap = {
    'Arizona Cardinals'    : 'ARI',
    'Atlanta Falcons'      : 'ATL',
    'Baltimore Ravens'     : 'BAL',
    'Buffalo Bills'        : 'BUF',
    'Carolina Panthers'    : 'CAR',
    'Chicago Bears'        : 'CHI',
    'Cincinnati Bengals'   : 'CIN',
    'Cleveland Browns'     : 'CLE',
    'Dallas Cowboys'       : 'DAL',
    'Denver Broncos'       : 'DEN',
    'Detroit Lions'        : 'DET',
    'Green Bay Packers'    : 'GB',
    'Houston Texans'       : 'HOU',
    'Indianapolis Colts'   : 'IND',
    'Jacksonville Jaguars' : 'JAX',
    'Kansas City Chiefs'   : 'KC',
    'Los Angeles Rams'     : 'LAR',
    'St. Louis Rams'       : 'LAR',
    'Los Angeles Chargers' : 'LAC',
    'Las Vegas Raiders'    : 'LV',
    'Oakland Raiders'      : 'LV',
    'Miami Dolphins'       : 'MIA',
    'Minnesota Vikings'    : 'MIN',
    'New England Patriots' : 'NE',
    'New Orleans Saints'   : 'NO',
    'New York Giants'      : 'NYG',
    'New York Jets'        : 'NYJ',
    'Philadelphia Eagles'  : 'PHI',
    'Pittsburgh Steelers'  : 'PIT',
    'Seattle Seahawks'     : 'SEA',
    'San Francisco 49ers'  : 'SF',
    'Tampa Bay Buccaneers' : 'TB',
    'Tennessee Titans'     : 'TEN',
    'Washington'           : 'WSH',
    'Washington Commanders': 'WSH',
    'Washington Redskins'  : 'WSH'
};

const GetTeamShortName = (longname) => {
    return lNameMap[longname];
};
