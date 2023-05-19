const port = 8080;

const express = require('express');
const app = express();
// Setup rendering engine
app.engine('html', require('ejs').renderFile);
app.set('view engine', 'html');

// Setup paths for Bootstrap
app.use(express.static(__dirname + '\\node_modules\\bootstrap\\dist'));
app.use(express.static('public'));

app.get('/', (_, res) => {
    res.render('picks.ejs');
});

app.listen(port, () => {
    console.log(`using -${__dirname + '\\node_modules\\bootstrap\\dist'}- for static files`);
    console.log(`Picks page application, listening on port ${port}`);
});
