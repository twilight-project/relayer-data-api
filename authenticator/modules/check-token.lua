local jwt = require "resty.jwt"
local header = ngx.var.http_authorization
local secret = require "jwt-secret"
local json = require 'cjson'

local token = string.gmatch(header, "(Bearer)[ ]+(.+)")
local t = {}
for k,v in token do
	t[k] = v
end

local jwt_obj = jwt:verify(secret, t["Bearer"])

if jwt_obj.valid ~= true then
	ngx.log(ngx.STDERR, "FORBIDDEN")
	return ngx.exit(ngx.HTTP_FORBIDDEN)
end

ngx.req.read_body()

local data = ngx.req.get_body_data()

local body = json.decode(data)
body["user"] = jwt_obj.payload

ngx.req.set_body_data(json.encode(body))
