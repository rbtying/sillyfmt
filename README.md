# `sillyfmt`
A naive formatter which pretty-prints text that looks vaguely like code. Unlike most formatters, `sillyfmt` does *not* require that the code parses "correctly", or that it fits some particular format. Instead, it tries the best it can to make sense of the input data.

Usage:

Run sillyfmt on its on to get interactive formatting:
```
[~] $ sillyfmt
Hit enter twice to format, or re-run with --newline
[{"_id":"5e345fc4179ff645f74b0c61","index":0,"guid":"81e5ad0e-2071-4d44-8720-7f02468cdadf","isActive":false,"balance":"$3,701.06","picture":"http://placehold.it/32x32","age":30,"eyeColor":"green","name":"Earnestine Bender","gender":"female","company":"EXOVENT","email":"earnestinebender@exovent.com","phone":"+1 (882) 427-2769","address":"876 Homecrest Court, Hall, Washington, 6511","about":"Aute dolor aute nostrud reprehenderit non commodo aliquip enim. Esse ad proident dolor exercitation laborum est labore est non Lorem adipisicing. Nulla ullamco id mollit proident.\r\n","registered":"2014-10-30T02:45:54 +07:00","latitude":17.48696,"longitude":167.668504,"tags":["adipisicing","eiusmod","culpa","dolor","duis","dolore","magna"],"friends":[{"id":0,"name":"Chandler Robinson"},{"id":1,"name":"Herrera Hess"},{"id":2,"name":"Elva Glass"}],"greeting":"Hello, Earnestine Bender! You have 10 unread messages.","favoriteFruit":"banana"},{"_id":"5e345fc401c64bb893ffe75b","index":1,"guid":"ff35ffeb-2a96-4c71-9f04-a624c3163b^C
robertying@st-robertying1 [~] $ sillyfmt
Hit enter twice to format, or re-run with --newline
[{"_id":"5e345fc4179ff645f74b0c61","index":0,"guid":"81e5ad0e-2071-4d44-8720-7f02468cdadf","isActive":false,"balance":"$3,701.06","picture":"http://placehold.it/32x32","age":30,"eyeColor":"green","name":"Earnestine Bender","gender":"female","company":"EXOVENT","email":"earnestinebender@exovent.com","phone":"+1 (882) 427-2769","address":"876 Homecrest Court, Hall, Washington, 6511","about":"Aute dolor aute nostrud reprehenderit non commodo aliquip enim. Esse ad proident dolor exercitation laborum est labore est non Lorem adipisicing. Nulla ullamco id mollit proident.\r\n","registered":"2014-10-30T02:45:54 +07:00","latitude":17.48696,"longitude":167.668504,"tags":["adipisicing","eiusmod","culpa","dolor","duis","dolore","magna"],"friends":[{"id":0,"name":"Chandler Robinson"},{"id":1,"name":"Herrera Hess"},{"id":2,"name":"Elva Glass"}],"greeting":"Hello, Earnestine Bender! You have 10 unread messages.","favoriteFruit":"banana"},{"_id":"5e345fc401c64bb893ffe75b","index":1,"guid":"ff35ffeb-2a96-4c71-9f04-a624c3163

[
  {
    "_id": "5e345fc4179ff645f74b0c61",
    "index": 0,
    "guid": "81e5ad0e-2071-4d44-8720-7f02468cdadf",
    "isActive": false,
    "balance": "$3,
    701.06",
    "picture": "http: //placehold.it/32x32",
    "age": 30,
    "eyeColor": "green",
    "name": "Earnestine Bender",
    "gender": "female",
    "company": "EXOVENT",
    "email": "earnestinebender@exovent.com",
    "phone": "+1 (882)427-2769",
    "address": "876 Homecrest Court,
    Hall,
    Washington,
    6511",
    "about": "Aute dolor aute nostrud reprehenderit non commodo aliquip enim. Esse ad proident dolor exercitation laborum est labore est non Lorem adipisicing. Nulla ullamco id mollit proident.\r\n",
    "registered": "2014-10-30T02: 45: 54 +07: 00",
    "latitude": 17.48696,
    "longitude": 167.668504,
    "tags":
    [
      "adipisicing",
      "eiusmod",
      "culpa",
      "dolor",
      "duis",
      "dolore",
      "magna"
    ],
    "friends":
    [
      {
        "id": 0,
        "name": "Chandler Robinson"
      },
      { "id": 1, "name": "Herrera Hess" },
      { "id": 2, "name": "Elva Glass" }
    ],
    "greeting": "Hello,
    Earnestine Bender! You have 10 unread messages.",
    "favoriteFruit": "banana"
  },
  {
    "_id": "5e345fc401c64bb893ffe75b",
    "index": 1,
    "guid": "ff35ffeb-2a96-4c71-9f04-a624c3163

]}
```

Run it with `--newline` to start formatting immediately, rather than waiting for an empty line.

You can also pipe data directly into `sillyfmt`.
