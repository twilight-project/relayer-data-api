local json = require 'cjson'

local h = ngx.req.get_headers()
local key = h["relayer-api-key"]
local sig = h["signature"]
local datetime = h["datetime"]

ngx.req.read_body()

local body = ngx.req.get_body_data()

local req = {}
req['api_key'] = key
req['sig'] = sig
req['datetime'] = datetime
req['body'] = body

local args = { method = ngx.HTTP_POST, body = json.encode(req) }
local resp = ngx.location.capture("/check", args)

if resp.status == 200 then
    local decoded = json.decode(body)
    local new_params = {}
    new_params["params"] = decoded["params"]
    new_params["user"] = json.decode(resp.body)
    decoded["params"] = new_params

    local jsonified = json.encode(decoded)
    ngx.req.set_body_data(jsonified)
else
    ngx.status = resp.status
    ngx.say(resp.body)
    ngx.exit(ngx.HTTP_OK)
end
