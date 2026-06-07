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
            let matchids = [];
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

                    match['homeRecordAll'] = hteam['records'][0];
                    match['homeRecordAlt'] = hteam['records'][1];
                    match['awayRecordAll'] = ateam['records'][0];
                    match['awayRecordAlt'] = ateam['records'][2];

                    if (m['id'] == feat_id) {
                        match['featured'] = true;
                    }

                    if (match['awayTeam'] === favteam || match['homeTeam'] === favteam) {
                        forcedid = m['id'];
                    }
                    matchids.push(m['id']);
                    return match;
                });
            }

            res.render('picks.html', {
                season, week,
                token,
                username, favteam, avatar,
                matches, matchids, forcedid,
                feat_val
            });
        });
    });
});

app.post('/submit/:token', (req, res) => {
    const token = req.params['token'];
    const matchids = req.body['matchids'];
    const favteam = req.body['favteam'];
    const forcedid = req.body['forcedid'];
    const feat_pick = req.body['feat_pick'];

    var picks = {};
    matchids.split(',').forEach((i) => {
        var pick = req.body[i];

        if (pick) {
            picks[i] = pick;
        } else if (forcedid === i) {
            picks[i] = favteam;
        } else {
            picks[i] = "N/A";
        }
    });

    LoadDB((db) => {
        // Resolve identity from the token, never from the request body
        db.get('SELECT poolerid, season, week FROM pick_tokens WHERE token = ?', token, (err, row) => {
            if (err || !row) {
                if (err) {
                    console.log(err);
                }
                res.render('error.html');
                return;
            }

            const { poolerid, season, week } = row;
            const insert = `
                INSERT INTO picks (season, week, poolerid, pickstring, featurepick)
                VALUES (?, ?, ?, ?, ?)
            `;
            db.run(insert, season, week, poolerid, JSON.stringify(picks), Number(feat_pick), (err) => {
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

app.get('/capsule/:discordid/:season', (req, res) => {
    const discordid = req.params['discordid'];
    const season = req.params['season'];

    LoadDB((db) => {
        const sql = `
            SELECT p.id AS poolerid,
                   (SELECT count(*) FROM capsules
                    WHERE poolerid = p.id AND season = ?) AS count
            FROM poolers AS p
            JOIN users AS u ON u.id = p.userid
            WHERE u.discordid = ?
        `;
        db.get(sql, season, discordid, async (err, row) => {
            if (err || !row) {
                console.log(err);
                res.render('error.html');
                return;
            }

            if (row['count'] != 0) {
                res.render('error.html');
                return;
            }

            res.render('playoffs.html', {
                afcTeams,
                nfcTeams,
                poolerid: row['poolerid'],
                season: season
            });
        });
    });
});

const CAP_TYPE = { DIVISION_WIN: 0, WILDCARD: 1 };
const CAP_CONF = { NFC: 0, AFC: 1 };
const DIV_ORDER = ['North', 'South', 'East', 'West']; // index = division column value

app.post('/capsule-submit', (req, res) => {
    const { poolerid, season, afcWinners, nfcWinners, afcWildcards, nfcWildcards } = req.body;

    // Parse the JSON strings from the form
    const afcWinnersObj = JSON.parse(afcWinners);
    const nfcWinnersObj = JSON.parse(nfcWinners);
    const afcWildcardsArr = JSON.parse(afcWildcards);
    const nfcWildcardsArr = JSON.parse(nfcWildcards);

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

    if (rows.length !== 14 || rows.some(r => !r[6])) {
        console.log('Incomplete capsule submission, refusing to insert');
        res.render('error.html');
        return;
    }

    LoadDB((db) => {
        const placeholders = rows.map(() => '(?, ?, ?, ?, ?, ?, ?)').join(', ');
        const sql = `
            INSERT INTO capsules (season, poolerid, type, conference, division, slot, team)
            VALUES ${placeholders}
        `;
        db.run(sql, rows.flat(), (err) => {
            if (err) {
                console.log(err);
                res.render('error.html');
            } else {
                res.render('success.html');
            }
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
