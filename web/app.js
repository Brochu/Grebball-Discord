const port = 8080;

const express = require('express');
const app = express();
// Setup rendering engine
app.engine('html', require('ejs').renderFile);
app.set('view engine', 'html');

// Setup paths for Bootstrap
app.use(express.static(__dirname + '\\node_modules\\bootstrap\\dist'));
app.use(express.static('public'));

app.get('/:season/:week/:token', (req, res) => {
    res.render('picks.html', { p: req.params, array: [ 1, 2, 3 ] });
});

app.listen(port, () => {
    console.log(`using -${__dirname + '\\node_modules\\bootstrap\\dist'}- for static files`);
    console.log(`Picks page application, listening on port ${port}`);
});
