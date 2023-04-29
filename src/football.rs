use serenity::model::id::EmojiId;

pub fn get_team_emoji(team: &str) -> EmojiId {
    return EmojiId(match team {
        "ARI" => 1101771924398424115,
        "ATL" => 1101771925853847582,
        "BAL" => 1101771926738829332,
        "BUF" => 1101771927724490843,
        "CAR" => 1101771928542400512,
        "CHI" => 1101771930069127188,
        "CIN" => 1101771930677301270,
        "CLE" => 1101771931868463125,
        "DAL" => 1101772137217400892,
        "DEN" => 1101772138664443914,
        "DET" => 1101772139545235456,
        "GB"  => 1101772244583206963,
        "HOU" => 1101771936352186388,
        "IND" => 1101772246214787072,
        "JAX" => 1101772247200440321,
        "KC"  => 1101771938868768840,
        "LA"  => 1101772371418947584,
        "LAC" => 1101772373029552189,
        "LV"  => 1101772373964882000,
        "MIA" => 1101771943377633361,
        "MIN" => 1101772510556602398,
        "NE"  => 1101772512121073664,
        "NO"  => 1101772512922181662,
        "NYG" => 1101772513782026381,
        "NYJ" => 1101772514662817843,
        "PHI" => 1101771950663155802,
        "PIT" => 1101772694015459408,
        "SEA" => 1101771953745965126,
        "SF"  => 1101772694875279521,
        "TB"  => 1101772696716574780,
        "TEN" => 1101772697580605522,
        "WAS" => 1101771957831221338,
        _     => 0,
    });
}