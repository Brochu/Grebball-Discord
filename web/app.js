const express = require('express');
const app = express();
const port = 8080;

app.engine('html', require('ejs').renderFile);
app.set('view engine', 'html');

app.get('/', (_, res) => {
    res.render('picks.ejs');
});

app.listen(port, () => {
    console.log(`Picks page application, listening on port ${port}`);
});
