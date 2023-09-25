local jwt = require "resty.jwt"
local header = ngx.var.http_authorization
local secret = os.getenv("NGINX_AUTH_SECRET")
local json = require 'cjson'

if header == nil or header == '' then
	ngx.log(ngx.STDERR, "NO BEARER TOKEN")
	return ngx.exit(ngx.HTTP_FORBIDDEN)
end

local token = string.gmatch(header, "(Bearer)[ ]+(.+)")
local t = {}
for k,v in token do
	t[k] = v
end

local jwt_obj = jwt:verify(secret, t["Bearer"])

if jwt_obj.valid ~= true then
    ngx.log(ngx.STDERR, "INVALID TOKEN")
    return ngx.exit(ngx.HTTP_BAD_REQUEST)
end

if jwt_obj.verified ~= true then
	ngx.log(ngx.STDERR, "FORBIDDEN")
	return ngx.exit(ngx.HTTP_FORBIDDEN)
end

ngx.req.read_body()

local data = ngx.req.get_body_data()

local body = json.decode(data)
local new_params = {}
new_params["params"] = body["params"]
new_params["user"] = jwt_obj.payload
body["params"] = new_params

local jsonified = json.encode(body)
ngx.req.set_body_data(jsonified)
