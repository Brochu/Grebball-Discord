const port = 8080;

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

app.get('/:season/:week/:pickid/:user/:id-:hash', async (req, res) => {
    const season = req.params['season'];
    const week = req.params['week'];
    const pickid = req.params['pickid'];
    const user = req.params['user'];
    const avatar = `${req.params['id']}/${req.params['hash']}`;

    const url = `https://www.thesportsdb.com/api/v1/json/3/eventsround.php?id=4391&r=${week}&s=${season}`;
    const result = await fetch(url);
    const json = await result.json();

    let matches = [];
    let matchids = [];
    if (json['events']) {
        matches = json['events'].map((m) => {
            m['awayTeam'] = GetTeamShortName(m['strAwayTeam']);
            m['homeTeam'] = GetTeamShortName(m['strHomeTeam']);

            matchids.push(m['idEvent']);
            return m;
        });
    }

    res.render('picks.html', { season, week, pickid, user, avatar, matches, matchids });
});

app.post('/submit', (req, res) => {
    const pickid = req.body['pickid'];
    const matchids = req.body['matchids'];

    console.log('Submitting picks at id: ', pickid);
    var picks = [];
    matchids.split(',').forEach((i) => {
        console.log('Match id: ', i);
        picks.push(req.body[i]);
    });
    console.log('Submitting picks : ', picks);

    res.send('OK');
});

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
