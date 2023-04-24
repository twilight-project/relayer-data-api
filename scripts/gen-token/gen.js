const fs = require('fs');
const jwt = require('jsonwebtoken');
const args = require('args-parser')(process.argv);

var secret;
var userid;
var is_admin;

if (args.secret !== undefined) {
	secret = fs.readFileSync('private.key');
} else {
	secret = 'test_secret';
}

if (args.userid !== undefined) {
	userid = args.userid;
} else {
	userid = 'test_user';
}

if (args.is_admin !== undefined) {
	is_admin = args.is_admin;
} else {
	is_admin = false;
}

// 1 hour from now, plenty good for testing
var expires = Math.floor(Date.now() / 1000) + (60 * 60);

console.log(jwt.sign({userid: userid, is_admin: is_admin, exp: expires }, secret))
