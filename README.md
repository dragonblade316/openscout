# Openscout Server
(might change the name)

Openscout is an easy-to-use backend for scouting apps.

Openscout is designed for multiple scouts from multiple teams to collaborate and share data. This makes life easier for teams that are not big enough to assign 6 members to scouting.
It is also designed to be flexible and adaptable to individual teams' needs.

## How it works

There are two kinds of reports, Pit Scout Reports and Single Team Match Scout Reports.
These are the only two structs that are guaranteed to change each year (requiring scouting app updates).

These data structures are defined as Rust structs and are translated to JSON with the Serde crate.

Some fields will be mandatory (if you do not fill them the server will return an internal error as it fails to translate the JSON to the rust struct).
These fields will contain basic data about the Team and match you are scouting and who you are (team and match numbers), as well as data fields that should realistically be in every scouting app.

If you need more data than what is provided by the default data fields, there will be a field in each type of report for team-specific data which will accept pretty much any valid json.

The server is written this way to be as easy to maintain as possible. 
The only task required to update the app from one year to another is updating the year-specific structures.
All other parts of the app will adapt.

This data can be read by any team who has access to the server (including the notes you make about the other teams, be nice nerds).


## Features and TODO

Easy to use RestApi: WIP.

The Blue Alliance and Statbotics integration: done.

Smart team assignments using FMS api: Auth problems.

MongoDB database: working.

Well-defined data structures: Done (probably).

JSON Schemas: Not started.

Per Team Auth: Not started (pretty easy I think).

Easy Toml Configuration: Not Started.

Callbacks and Webhooks: I will work on this if I have time and Axum supports it.

Documentation: I have not figured out what this will look like yet.

## Versioning Scheme

The version is split into 4 parts, Season, Major, Minor, and Tag

Versions of openscout from different seasons are not compadible as the season spesific datatypes have been replaced.


Season will change every season (hopefully this was obvious)

A change of the major version implies breaking changes to the api or significant new fetures.

A change to the minor version implies bug fixes or fetures that will not break the current api.

The tag will either be WIP, Alpha, Beta, Nightly, and Stable. This will indicate how confident I am that everything will work. WIP and Alpha will likely never be seen again after the 1st beta.

## FAQ

Will there be an official openscout client: Probably not, the project is designed to require as little effort to maintain as possible and dealing with a changing UI would go against that. However, I am hoping to have at least one team make their scouting app open source. This will mean that teams should have access to an Openscout client even if they dod not have the resources to make their own.

Can we prevent other teams from querying our scouting data: By hosting your own instance and not allowing other teams to access it, yes. However, the server itself will (most likely) not support any form of access control. WIP

How do you decide the default fields: Mostly guessing and asking teams. Major version changes may change the schema to make it more useful for teams at the cost of teams having to update their scouting apps.

Did anyone actually ask: No, I made these up.

## Contributing
If you want to contribute to this project feel free to open an issue, submit a pull request, or contact me.
You will be able to find me on Cheif Delphi and the FRC Scouting and Statagy discord server. (I am dragonblade316 on both)
