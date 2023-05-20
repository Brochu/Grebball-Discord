const port = 8080;

const express = require('express');
const app = express();
// Setup rendering engine
app.engine('html', require('ejs').renderFile);
app.set('view engine', 'html');

// Setup paths for Bootstrap
app.use(express.static(__dirname + '\\node_modules\\bootstrap\\dist'));
app.use(express.static('public'));

app.get('/:season/:week/:token', async (req, res) => {
    const season = req.params['season']
    const week = req.params['week']
    const url = `https://www.thesportsdb.com/api/v1/json/3/eventsround.php?id=4391&r=${week}&s=${season}`;

    const result = await fetch(url);
    const json = await result.json();
    const events = json['events']

    res.render('picks.html', { season, week, matches: events });
});

app.listen(port, () => {
    console.log(`using -${__dirname + '\\node_modules\\bootstrap\\dist'}- for static files`);
    console.log(`Picks page application, listening on port ${port}`);
});
