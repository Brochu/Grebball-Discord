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
            SELECT u.avatar, po.name, po.favteam, pi.season, pi.week, pi.pickstring, ft.type, ft.target, ft.match
            FROM users AS u
                JOIN poolers as po
                ON u.id = po.userid
                JOIN picks as pi
                ON po.id = pi.poolerid
                LEFT JOIN features as ft
                ON ft.season = pi.season AND ft.week = pi.week
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

                const feat_id = row['match'];
                const feat_type = row['type'];
                const feat_val = row['target'];

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
                        match = {};
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
                    pickid,
                    username, favteam, avatar,
                    matches, matchids, forcedid,
                    feat_val
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
        const sql = `
            UPDATE picks
                SET pickstring = ?, featurepick = ?
            WHERE id = ?
        `;
        db.run(sql, JSON.stringify(picks), Number(feat_pick), pickid, (err) => {
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

app.get('/capsule/:discordid/:season', (req, res) => {
    const discordid = req.params['discordid'];
    const season = req.params['season'];

    LoadDB((db) => {
        const sql = `
            SELECT c.poolerid, c.season, c.winafcn, c.winafcs, c.winafce, c.winafcw,
                   c.winnfcn, c.winnfcs, c.winnfce, c.winnfcw, c.afcwildcards, c.nfcwildcards
            FROM capsules AS c
                JOIN poolers AS p ON p.id = c.poolerid
                JOIN users AS u ON u.id = p.userid
            WHERE u.discordid = ? AND c.season = ?
        `;
        db.get(sql, discordid, season, async (err, row) => {
            if (err || !row) {
                console.log(err);
                res.render('error.html');
                return;
            }

            // Check if picks have already been submitted (any division winner is not null)
            if (row['winafcn'] != null || row['winafcs'] != null ||
                row['winafce'] != null || row['winafcw'] != null ||
                row['winnfcn'] != null || row['winnfcs'] != null ||
                row['winnfce'] != null || row['winnfcw'] != null) {
                res.render('error.html');
                return;
            }

            res.render('playoffs.html', {
                afcTeams,
                nfcTeams,
                poolerid: row['poolerid'],
                season: row['season']
            });
        });
    });
});

app.post('/capsule-submit', (req, res) => {
    const { poolerid, season, afcWinners, nfcWinners, afcWildcards, nfcWildcards } = req.body;

    // Parse the JSON strings from the form
    const afcWinnersObj = JSON.parse(afcWinners);
    const nfcWinnersObj = JSON.parse(nfcWinners);
    const afcWildcardsArr = JSON.parse(afcWildcards);
    const nfcWildcardsArr = JSON.parse(nfcWildcards);

    // Convert full team names to short names
    const afcWildcardsShort = afcWildcardsArr.map(GetTeamShortName).join(',');
    const nfcWildcardsShort = nfcWildcardsArr.map(GetTeamShortName).join(',');

    LoadDB((db) => {
        const sql = `
            UPDATE capsules
            SET winafcn = ?, winafcs = ?, winafce = ?, winafcw = ?,
                winnfcn = ?, winnfcs = ?, winnfce = ?, winnfcw = ?,
                afcwildcards = ?, nfcwildcards = ?
            WHERE poolerid = ? AND season = ?
        `;
        db.run(sql,
            GetTeamShortName(afcWinnersObj['North']),
            GetTeamShortName(afcWinnersObj['South']),
            GetTeamShortName(afcWinnersObj['East']),
            GetTeamShortName(afcWinnersObj['West']),
            GetTeamShortName(nfcWinnersObj['North']),
            GetTeamShortName(nfcWinnersObj['South']),
            GetTeamShortName(nfcWinnersObj['East']),
            GetTeamShortName(nfcWinnersObj['West']),
            afcWildcardsShort, nfcWildcardsShort,
            poolerid, season,
            (err) => {
                if (err) {
                    console.log(err);
                    res.render('error.html');
                } else {
                    res.render('success.html');
                }
            }
        );
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
