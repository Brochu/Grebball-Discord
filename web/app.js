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

app.get('/playoffs/:discordid/:season', (req, res) => {
    const discordid = req.params['discordid'];
    const season = req.params['season'];

    LoadDB((db) => {
        const sql = `
            SELECT discordid, season FROM capsules AS c
                JOIN poolers AS p
                ON p.id == c.poolerid
                JOIN users AS u
                ON u.id == p.userid
            WHERE u.discordid = ? AND season = ?
        `;
        db.get(sql, discordid, season, async (err, row) => {
            if (err || !row) {
                console.log(err);
                res.render('error.html');
            }
            else {
                const test0 = row['discordid'];
                console.log(test0);
                const test1 = row['season'];
                console.log(test1);

                res.render('playoffs.html', {
                    afcTeams,
                    nfcTeams
                });
            }
        });
    });
});

app.post('/submit-playoffs', (req, res) => {
});

const port = 3000;

app.listen(port, () => {
    console.log(`Picks page application, listening on port ${port}`);
});

const afcTeams = [
  { name: 'Buffalo Bills', division: 'East', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/buf.png' },
  { name: 'Miami Dolphins', division: 'East', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/mia.png' },
  { name: 'New York Jets', division: 'East', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/nyj.png' },
  { name: 'New England Patriots', division: 'East', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/ne.png' },
  { name: 'Baltimore Ravens', division: 'North', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/bal.png' },
  { name: 'Cincinnati Bengals', division: 'North', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/cin.png' },
  { name: 'Cleveland Browns', division: 'North', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/cle.png' },
  { name: 'Pittsburgh Steelers', division: 'North', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/pit.png' },
  { name: 'Houston Texans', division: 'South', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/hou.png' },
  { name: 'Indianapolis Colts', division: 'South', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/ind.png' },
  { name: 'Jacksonville Jaguars', division: 'South', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/jax.png' },
  { name: 'Tennessee Titans', division: 'South', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/ten.png' },
  { name: 'Kansas City Chiefs', division: 'West', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/kc.png' },
  { name: 'Las Vegas Raiders', division: 'West', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/lv.png' },
  { name: 'Los Angeles Chargers', division: 'West', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/lac.png' },
  { name: 'Denver Broncos', division: 'West', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/den.png' }
];

const nfcTeams = [
  { name: 'Dallas Cowboys', division: 'East', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/dal.png' },
  { name: 'Philadelphia Eagles', division: 'East', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/phi.png' },
  { name: 'New York Giants', division: 'East', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/nyg.png' },
  { name: 'Washington Commanders', division: 'East', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/wsh.png' },
  { name: 'Detroit Lions', division: 'North', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/det.png' },
  { name: 'Green Bay Packers', division: 'North', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/gb.png' },
  { name: 'Minnesota Vikings', division: 'North', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/min.png' },
  { name: 'Chicago Bears', division: 'North', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/chi.png' },
  { name: 'Tampa Bay Buccaneers', division: 'South', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/tb.png' },
  { name: 'Atlanta Falcons', division: 'South', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/atl.png' },
  { name: 'New Orleans Saints', division: 'South', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/no.png' },
  { name: 'Carolina Panthers', division: 'South', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/car.png' },
  { name: 'San Francisco 49ers', division: 'West', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/sf.png' },
  { name: 'Los Angeles Rams', division: 'West', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/lar.png' },
  { name: 'Seattle Seahawks', division: 'West', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/sea.png' },
  { name: 'Arizona Cardinals', division: 'West', logo: 'https://a.espncdn.com/i/teamlogos/nfl/500/ari.png' }
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
