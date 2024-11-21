# Openscout Server
(might change the name)

Openscout is an easy* to use backend for scouting apps.

Openscout is designed to for multiple scouts from multiple teams to collaberate and share data. This makes life easier for teams who are not big enough to assign 6 members to scouting.
It is also designed to flexable and adaptable to induvidual teams needs.

## How it works

There are two kinds of report, Pit Scout Reports and Single Team Match Scout Reports.
These are the only two structs that are gurenteed to change each year (requiring scouting app updates).

These data structures are defined as Rust structs and are traslated to json with the serde crate.

Some fields will be mandatory (if you do not fill them the server will return an internal error as it fails to traslate to the rust struct).
These fields will be basic data about the Team and match you are scouting and who you are (team and match numbers) as well as data fields that realisticly should be in every scouting app.

If you need more data than what is provided by the default data fields, there will be a field in each type of report for team spesific data which will accept pretty much any valid json.

The server is writen this way in order to make it as easy to maintain as possible. 
The only task required to update the app from one year to another is updating the year spesific structures.
All other parts of the app will adapt.

This data can be read by any team who has access to the server (including the notes you make about the other teams, be nice nerds).


## Features and TODO

Easy to use RestApi: WIP.
The Blue Allience and statbotics intergration: done.
Smart team assignments using FMS api: Auth problems.
MongoDB database: working.
Well defined data structures: WIP.
JSON Schemas: Not started.
Per Team Auth: Not started (pretty easy I think).
Easy Toml Configuration: Not Started.
Callbacks and Webhooks: Will work on this if I have time and axum supports it.
Documentation: Have not figured out what this will look like yet.
A Brilliantly writen readme: lol, this probably has at least 10 grammer mistakes.

## Versioning Scheme

WIP

## FAQ

Will there be an offical openscout client: Probably not, the project is designed to require as little effort to maintain as possible and dealing with a changing UI would go against that. However, I am hoping to have at least one team make their scouting app open source. This will mean that teams should have access to an Openscout client even if they dod not have the resources to make their own.

Can we prevent other teams from qurrying our scouting data: By hosting your own instance and not allowing other teams to access it, yes. But the server itself will (most likely) not support any form of access control. 

How do you decide the default fields: Mostly guessing and asking teams. Major version changes may change the schema to make it more useful for teams at the cost of teams having to update their scouting apps.

Did anyone actually ask: No, I made these up.
