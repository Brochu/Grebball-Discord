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

app.get('/:discordid/:pickid', async (req, res) => {
    // Get params from querystring
    const discordid = req.params['discordid'];
    const pickid = req.params['pickid'];

    LoadDB((db) => {
        const sql = `
            SELECT u.avatar, po.name, po.favteam, pi.season, pi.week, pi.pickstring
            FROM users AS u
                JOIN poolers as po
                ON u.id = po.userid
                JOIN picks as pi
                ON po.id = pi.poolerid
            WHERE u.discordid = ? AND pi.id = ?
        `;
        db.get(sql, discordid, pickid, async (err, row) => {
            if (err || !row) {
                if (err) {
                    console.log('Could not query DB for users, err: ', err.message);
                }
                res.render('error.html');
                return;
            }
            else {
                // Picks are already in the DB
                if (row['pickstring'] != null) {
                    res.render('error.html');
                    return;
                }

                const avatar = row['avatar'];
                const username = row['name'];
                const favteam = row['favteam']
                const season = row['season'];
                const week = row['week'];

                var w = week;
                if (week == 19 || week == '19') {
                    w = 160;
                }
                else if (week == 20 || week == '20') {
                    w = 125;
                }
                else if (week == 21 || week == '21') {
                    w = 150;
                }
                else if (week == 22 || week == '22') {
                    w = 200;
                }
                const url = `https://www.thesportsdb.com/api/v1/json/3/eventsround.php?id=4391&r=${week}&s=${season}`;
                const result = await fetch(url);
                const json = await result.json();

                let matches = [];
                let matchids = [];
                let forcedid = 0;
                if (json['events']) {
                    matches = json['events'].map((m) => {
                        m['awayTeam'] = GetTeamShortName(m['strAwayTeam']);
                        m['homeTeam'] = GetTeamShortName(m['strHomeTeam']);

                        if (m['awayTeam'] === favteam || m['homeTeam'] === favteam) {
                            forcedid = m['idEvent'];
                        }
                        matchids.push(m['idEvent']);
                        return m;
                    });
                }

                res.render('picks.html', {
                    season,
                    week,
                    pickid,
                    username,
                    favteam,
                    avatar,
                    matches,
                    matchids,
                    forcedid
                });
            }
        });
    });
});

app.post('/submit', (req, res) => {
    const pickid = req.body['pickid'];
    const matchids = req.body['matchids'];
    const favteam = req.body['favteam'];
    const forcedid = req.body['forcedid'];

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
        const sql = `
            UPDATE picks
                SET pickstring = ?
            WHERE id = ?
        `;
        db.run(sql, JSON.stringify(picks), pickid, (err) => {
            if (err) {
                console.log(err);
                res.render('error.html');
            }
            else {
                res.render('success.html');
            }
        });
    });
});

const port = 3000;

app.listen(port, () => {
    console.log(`Picks page application, listening on port ${port}`);
});

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
    'Los Angeles Rams'     : 'LA',
    'St. Louis Rams'       : 'LA',
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
    'Washington'           : 'WAS',
    'Washington Commanders': 'WAS',
    'Washington Redskins'  : 'WAS'
};

const GetTeamShortName = (longname) => {
    return lNameMap[longname];
};
