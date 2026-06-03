BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS "capsules" (
	"id"	     INTEGER PRIMARY KEY AUTOINCREMENT,
	"season"	 INTEGER NOT NULL DEFAULT 2000,
	"poolerid"	 INTEGER,

	"type"       INTEGER,
	"conference" INTEGER,
	"division"   INTEGER,
	"slot"       INTEGER,
	"team"       TEXT,
	CONSTRAINT "PoolerId_FK" FOREIGN KEY("poolerid") REFERENCES "poolers"("id")
);
CREATE TABLE IF NOT EXISTS "features" (
	"id"	INTEGER,
	"season"	INTEGER NOT NULL DEFAULT (2000),
	"week"	INTEGER NOT NULL DEFAULT (1),
	"type"	INTEGER,
	"target"	INTEGER,
	"match"	TEXT,
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE TABLE IF NOT EXISTS "picks" (
	"id"	INTEGER,
	"season"	INTEGER,
	"week"	INTEGER,
	"pickstring"	TEXT,
	"poolerid"	INTEGER,
	"scorecache"	INTEGER,
	"featurepick"	INTEGER,
	"featcache"	INTEGER,
	PRIMARY KEY("id" AUTOINCREMENT),
	CONSTRAINT "PoolerId_FK" FOREIGN KEY("poolerid") REFERENCES "poolers"("id") ON DELETE SET NULL
);
CREATE TABLE IF NOT EXISTS "poolers" (
	"id"	INTEGER,
	"name"	TEXT,
	"favteam"	TEXT,
	"poolid"	INTEGER,
	"userid"	INTEGER,
	PRIMARY KEY("id" AUTOINCREMENT),
	CONSTRAINT "PoolId_FK" FOREIGN KEY("poolid") REFERENCES "pools"("id") ON DELETE SET NULL,
	CONSTRAINT "UserId_FK" FOREIGN KEY("userid") REFERENCES "users"("id") ON DELETE SET NULL
);
CREATE TABLE IF NOT EXISTS "pools" (
	"id"	INTEGER,
	"name"	TEXT,
	"motp"	TEXT,
	PRIMARY KEY("id" AUTOINCREMENT)
);
CREATE TABLE IF NOT EXISTS "users" (
	"id"	INTEGER,
	"email"	TEXT UNIQUE,
	"access"	INTEGER,
	"discordid"	INTEGER UNIQUE,
	"avatar"	TEXT,
	PRIMARY KEY("id" AUTOINCREMENT)
);

COMMIT;
