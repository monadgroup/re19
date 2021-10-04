local M = {}

local function str_split(str, pat)
	local t = {} 
	local fpat = "(.-)" .. pat
	local last_end = 1
	local s, e, cap = str:find(fpat, 1)
	while s do
		table.insert(t,cap)
		last_end = e+1
		s, e, cap = str:find(fpat, last_end)
	end
	if last_end <= #str then
		cap = str:sub(last_end)
		table.insert(t, cap)
	elseif str:sub(-1)==pat then
		table.insert(t, "")
	end
	return t
end
local function split_comment(line)
    local comment = line:match("(%s*//.*)") or ""
    line = line:gsub("%s*//.*","")
    line = line:gsub("%s*$","")
    return line,comment
end
M.split_comment = split_comment
local function strip(cad)
    return cad:gsub("^%s*(.-)%s*$","%1") --remove initial and final spaces
end
local function strip_end(cad)
    return cad:gsub("^(.-)%s*$","%1") --remove initial and final spaces
end
local function clean_spaces(cad)
    cad = strip(cad)
    cad = cad:gsub("%s+"," ") --not more than one space
    cad = cad:gsub("%s*([%(%),=])%s*","%1") --not spaces with ( , )
    return cad
end
function strsplit(str, pat)
    local t = {} 
    local fpat = "(.-)" .. pat
    local last_end = 1
    local s, e, cap = str:find(fpat, 1)
    while s do
        table.insert(t,cap)
        last_end = e+1
        s, e, cap = str:find(fpat, last_end)
    end
    if last_end <= #str then
        cap = str:sub(last_end)
        table.insert(t, cap)
    elseif str:sub(-1)==pat then
        table.insert(t, "")
    end
    return t
end
--------------------------------------------------------------------------
local function save_data(filename,...)
    local file,err = io.open(filename,"w")
    if not file then error(err) end
    for i=1, select('#', ...) do
        local data = select(i, ...)
        file:write(data)
    end
    file:close()
end
M.save_data = save_data
----------------------------------------
local function  read_data(filename)
    local hfile,err = io.open(filename,"r")
    if not hfile then error(err) end
    local hstrfile = hfile:read"*a"
    hfile:close()
    return hstrfile
end
M.read_data = read_data
------------------------------------------------------------
local function copyfile(src,dst,blocksize)
    blocksize = blocksize or 1024*4
    print( "copyfile", src, dst)
    local srcf, err = io.open(src,"rb")
    if not srcf then error(err) end
    local dstf, err = io.open(dst,"wb")
    if not dstf then error(err) end
    while true do
        local data = srcf:read(blocksize)
        if not data then break end
        dstf:write(data)
    end
    srcf:close()
    dstf:close()
end
M.copyfile = copyfile

--gives the re table
local function getRE()
	local res = {
	function_re = "^([^;{}]+%b()[\n%s]*;)%s*",
	function_re = "^([^;{}]+%b()[\n%s%w]*;)", --const at the end
	struct_re = "^([^;{}]-struct[^;{}]-%b{}[%s%w_%(%)]*;)",
	enum_re = "^([^;{}]-enum[^;{}]-%b{}[%s%w_%(%)]*;)",
	union_re = "^([^;{}]-union[^;{}]-%b{}[%s%w_%(%)]*;)",
	structenum_re = "^([^;{}]-%b{}[%s%w_%(%)]*;)",
	namespace_re = "^([^;{}]-namespace[^;{}]-%b{})",
	class_re = "^([^;{}]-class[^;{}]-%b{}%s*;)",
	typedef_re = "^\n*(typedef[^;]+;)",
	functypedef_re = "^\n*%s*(typedef[%w%s%*_]+%(%s*%*%s*[%w_]+%s*%)%s*%b()%s*;)",
	functypedef_re = "^\n*%s*(typedef[%w%s%*_]+%([^*]*%*%s*[%w_]+%s*%)%s*%b()%s*;)",
	vardef_re = "^\n*([^;{}%(%)]+;)",
	functionD_re = "^([^;{}]-%b()[\n%s%w]*%b{})",
	--functionD_re = "^([^;{}]-%b()[^{}%(%)]*%b{})",
	functype_re = "^%s*[%w%s%*]+%(%*[%w_]+%)%([^%(%)]*%)%s*;"
	}
	
	local resN = {"functypedef_re","functype_re","function_re","functionD_re","struct_re","enum_re","union_re","namespace_re","class_re","typedef_re","vardef_re"}
	
	return res,resN
end
M.getRE = getRE
--takes preprocesed file in table cdefsor and returns items
local function parseItems(txt,dumpit)
	--dumpit = true
	local res,resN = getRE()
	
	local ini = 1
	local item
	local items = {}
	local itemarr = {}
	while true do
		local found = false
		for ire,re_name in ipairs(resN) do
			local re = res[re_name]
			local i,e = txt:find(re,ini)
			if i then
				item = txt:sub(i,e)
				--if re~=functionD_re then --skip defined functions
					item = item:gsub("extern __attribute__%(%(dllexport%)%) ","")
					table.insert(itemarr,{re_name=re_name,item=item})
				--end
				items[re_name] = items[re_name] or {}
				table.insert(items[re_name],item)
				found = true
				ini = e + 1
				if dumpit then
					print(item)
					print(ire,"------------------------------------------------------")
				end
				break
			end
		end
		if not found then
			if not (ini >= #txt) then 
				local rest = txt:sub(ini)
				local onlyspaces = rest:match("^%s*$")
				if not onlyspaces then
					print(ini,#txt);
					print(txt);
					print(item)
					error"parseItems error"
				end
			end
			break 
		end
	end
	return itemarr,items
end
M.parseItems = parseItems
local function name_overloadsAlgo(v)
    local aa = {}
    local bb = {}
    local done = {}
    local maxnum = 0
    for i,t in ipairs(v) do
        bb[i] = ""
        local signature = t.signature:sub(2,-2) -- without parenthesis
        aa[i] = {}
        local num = 1
        --for typec in t.signature:gmatch(".-([^,%(%s]+)[,%)]") do
        --for typec in t.signature:gmatch(".-([^,%(]+)[,%)]") do
        --for typec in signature:gmatch(".-([^,]+),?") do
        for typec in signature:gsub("(%(.-%))", function(x) return x:gsub(",","\0") end):gmatch(".-([^,]+),?") do
            --typec = typec:gsub
            aa[i][num] = typec:gsub("%z+", ",")
            num = num + 1
        end
        num = num - 1
        maxnum = (num > maxnum) and num or maxnum
    end
    
    for l=1,maxnum do
        local keys = {}
        local diferent = true
        local equal = true
        for i=1,#v do
            aa[i][l] = aa[i][l] or "nil"
            keys[aa[i][l]] = 1 + (aa[i][l] and keys[aa[i][l]] or 0)
            if not done[i] then
            for j=i+1,#v do
                if not done[j] then
                if aa[i][l] == aa[j][l] then
                    diferent = false
                else
                    equal = false
                end
                end
            end
            end
        end
        if not equal then -- not all the same
            for i=1,#v do
                if not done[i] then
                    bb[i] = bb[i]..(aa[i][l]=="nil" and "" or aa[i][l])
                    if keys[aa[i][l]] == 1 then
                        done[i] = true
                    end
                end
            end
        end
    end
    return aa,bb
end
local function typetoStr(typ)
    --typ = typ:gsub("[^%(%)]+%(%*?(.+)%).+","%1") -- funcs
    typ = typ:gsub("[^%(%)]+%(%*?(.+)%).+","FnPtr")
    typ = typ:gsub("[%w_]+%[(%d*)%]","arr%1")
    typ = typ:gsub("%*","Ptr")
    typ = typ:gsub("void","")
    typ = typ:gsub("unsigned%s","u")
    typ = typ:gsub("const%s","")--"c")
    typ = typ:gsub("%s+","_")
    typ = typ:gsub("charPtr","Str")
    typ = typ:gsub("int","Int")
    typ = typ:gsub("bool","Bool")
    typ = typ:gsub("float","Float")
    typ = typ:gsub("uInt","Uint")
    typ = typ:gsub("ImGui","")
    typ = typ:gsub("Im","")
    typ = typ:gsub("[<>]","")
    return typ
end
local function parseFunction(self,stname,lineorig,namespace)
	line = clean_spaces(lineorig)
	--move *
	line = line:gsub("%s*%*","%*")
	line = line:gsub("%*([%w_])","%* %1")
	line = line:gsub("(%(%*)%s","%1")

	--print(line)
    --clean implemetation
    line = line:gsub("%s*%b{}","")
    --clean attribute
    line = line:gsub("%s*__attribute__%b()","")
    --clean static and inline and mutable
    line = line:gsub("static","")
	line = line:gsub("inline","")
	line = line:gsub("mutable","")
	--skip operator
	if line:match("operator") then return end
	--skip template
	if line:match("template") then return end
    
    local ret = line:match("([^%(%)]+[%*%s])%s?~?[_%w]+%b()")
    local funcname, args = line:match("(~?[_%w]+)%s*(%b())")
	
	if not args then
	print"not gettint args in"
	print(line,lineorig)
	print(funcname,"args",args)
    end
	
    local argscsinpars = args:gsub("(=[^,%(%)]*)(%b())","%1")
    argscsinpars = argscsinpars:gsub("(=[^,%(%)]*)([,%)])","%2")
    -- if argscsinpars:match("&") then 
        -- for arg in argscsinpars:gmatch("[%(,]*([^,%(%)]+)[%),]") do
            -- if arg:match("&") and not arg:match("const") then
                -- print(funcname,argscsinpars)
            -- end
        -- end
    -- end
    --argscsinpars = argscsinpars:gsub("&","")

	--for _,ttype in ipairs(self.templatedTypes) do
    --local template = argscsinpars:match(ttype.."%s*<(.+)>")
	local ttype,template = argscsinpars:match("([^%s,%(%)]+)%s*<(.+)>")
	local te=""
	if template then
		te = template:gsub("%s","_")
        te = te:gsub("%*","Ptr")
		self.templates[ttype] = self.templates[ttype] or {}
		self.templates[ttype][template] = te
	end
	--end
    argscsinpars = argscsinpars:gsub("<([%w_%*]+)>","_"..te) --ImVector
    
    local argsArr = {}
    local functype_re =       "^%s*[%w%s%*]+%(%*[%w_]+%)%([^%(%)]*%)"
    local functype_reex =     "^(%s*[%w%s%*]+)%(%*([%w_]+)%)(%([^%(%)]*%))"
    local functype_arg_rest = "^(%s*[%w%s%*]+%(%*[%w_]+%)%([^%(%)]*%)),*(.*)"
    local rest = argscsinpars:sub(2,-2) --strip ()
    
    while true do
    --local tt = strsplit(rest,",")
    --for ii,arg in ipairs(tt) do
    --for arg in argscsinpars:gmatch("[%(,]*([^,%(%)]+)[%),]") do
		if rest == "void" then break end
        local type,name,retf,sigf
        local arg,restt = rest:match(functype_arg_rest)
        if arg then
            local t1,namef,t2 = arg:match(functype_reex)
            type=t1.."(*)"..t2;name=namef
            retf = t1
            sigf = t2
            rest = restt
        else
            arg,restt = rest:match(",*([^,%(%)]+),*(.*)")
            if not arg then break end
            rest = restt
            if arg:match("&") and arg:match("const") then
                arg = arg:gsub("&","")
            end
            if arg:match("%.%.%.") then 
                type="...";name="..."
            else
                type,name = arg:match("(.+)%s([^%s]+)")
            end

            if not type or not name then 
                print("failure arg detection",funcname,type,name,argscsinpars,arg)

            else
				if name:match"%*" then print("**",funcname) end
                --float name[2] to float[2] name
                local siz = name:match("(%[%d*%])")
                if siz then
                    type = type..siz
                    name = name:gsub("(%[%d*%])","")
                end
            end
        end
        table.insert(argsArr,{type=type,name=name,ret=retf,signature=sigf})
        if arg:match("&") and not arg:match("const") then
            --only post error if not manual
            local cname = self.getCname(stname,funcname) --cimguiname
            if not self.manuals[cname] then
                print("reference to no const arg in",funcname,argscsinpars)
            end
        end
    end
    argscsinpars = argscsinpars:gsub("&","")
    
    local signature = argscsinpars:gsub("([%w%s%*_]+)%s[%w_]+%s*([,%)])","%1%2")
    signature = signature:gsub("%s*([,%)])","%1") --space before , and )
    signature = signature:gsub(",%s*",",")--space after ,
    signature = signature:gsub("([%w_]+)%s[%w_]+(%[%d*%])","%1%2") -- float[2]
    signature = signature:gsub("(%(%*)[%w_]+(%)%([^%(%)]*%))","%1%2") --func defs
    
    local call_args = argscsinpars:gsub("([%w_]+%s[%w_]+)%[%d*%]","%1") --float[2]
    call_args = call_args:gsub("%(%*([%w_]+)%)%([^%(%)]*%)"," %1") --func type
    call_args = call_args:gsub("[^%(].-([%w_]+)%s*([,%)])","%1%2")
    
    if not ret and stname then --must be constructors
        if not (stname == funcname or "~"..stname==funcname) then --break end
            print("false constructor:",line);
            print("b2:",ret,stname,funcname,args)
            return --are function defs
        end
    end
    
    local cimguiname = self.getCname(stname,funcname)
    table.insert(self.funcdefs,{stname=stname,funcname=funcname,args=args,argsc=argscsinpars,signature=signature,cimguiname=cimguiname,call_args=call_args,ret =ret,comment=comment})
	local defsT = self.defsT
	            defsT[cimguiname] = defsT[cimguiname] or {}
                table.insert(defsT[cimguiname],{})
                local defT = defsT[cimguiname][#defsT[cimguiname]] 
    defT.defaults = {}
    --for k,def in args:gmatch("([%w%s%*_]+)=([%w_%(%)%s,%*]+)[,%)]") do
    --for k,def in args:gmatch("([%w_]+)=([%w_%(%)%s,%*%.%-]+)[,%)]") do
    for k,def in args:gmatch('([%w_]+)=([%w_%(%)%s,%*%.%-%+%%"]+)[,%)]') do
        defT.defaults[k]=def
    end
	defT.namespace = namespace
    defT.cimguiname = cimguiname
    defT.stname = stname
    defT.funcname = funcname
    defT.argsoriginal = args
    defT.args=argscsinpars
    defT.signature = signature
    defT.call_args = call_args
    defT.isvararg = signature:match("%.%.%.%)$")
    defT.location = locat
    defT.comment = "" --comment
    defT.argsT = argsArr
    if self.get_manuals(defT) then
        defT.manual = true
    end
    if ret then
        defT.ret = clean_spaces(ret:gsub("&","*"))
        defT.retref = ret:match("&")
        -- if defT.ret=="ImVec2" or defT.ret=="ImVec4" or defT.ret=="ImColor" then
            -- defT.ret = defT.ret.."_Simple"
        -- end
    end
	defsT[cimguiname][signature] = defT
end
local function itemsCount(items)
	print"------------items"
	local res,resN = getRE()
	for k,v in pairs(resN) do
		local its = items[v]
		print(v,its and #its or 0)
	end
end

local function AdjustArguments(FP)
    for fun,defs in pairs(FP.defsT) do
        --struct function but no constructors
        if defs[1].stname~="" and defs[1].ret then
            --print("adjusting",fun)
            for i,def in ipairs(defs) do
                local empty = def.args:match("^%(%)") --no args
                --local ptret = def.retref and "&" or ""
                def.args = def.args:gsub("^%(","("..def.stname.."* self"..(empty and "" or ","))
                table.insert(def.argsT,1,{type=def.stname.."*",name="self"})
            end
        end
    end
end
local function ADDnonUDT(FP)
    local defsT = FP.defsT
    local newcdefs = {}
    --for cimguiname,defs in pairs(defsT) do
    --for i,defT in ipairs(defs) do
		--local t = {cimguiname=cimguiname,signature=defT.signature,ret=defT.ret}
    for numcdef,t in ipairs(FP.funcdefs) do
        if t.cimguiname then
        local cimf = defsT[t.cimguiname]
        local defT = cimf[t.signature]
        --if UDT return generate nonUDT version
		local isUDT = false
		for _,udt_ret in ipairs(FP.UDTs) do
			if udt_ret == defT.ret then isUDT=true;break end
		end
        --if defT.ret=="ImVec2" or defT.ret=="ImVec4" or defT.ret=="ImColor" then
		if isUDT then
            --passing as a pointer arg
            local defT2 = {}
            --first strings
            for k,v in pairs(defT) do
                defT2[k] = v
            end
            --then argsT table
            defT2.argsT = {{type=defT.ret.."*",name="pOut"}}
            for k,v in ipairs(defT.argsT) do
                table.insert(defT2.argsT,{type=v.type,name=v.name})
            end
            local comma = (#defT.argsT > 0) and "," or ""
            defT2.args = "("..defT.ret.." *pOut"..comma..defT.args:sub(2)
            defT2.ret = "void"
            defT2.ov_cimguiname = (defT2.ov_cimguiname or defT2.cimguiname).."_nonUDT"
            defT2.nonUDT = 1
            defT2.retref = nil
            defsT[t.cimguiname][#defsT[t.cimguiname] + 1] = defT2
            defsT[t.cimguiname][t.signature.."nonUDT"] = defT2
            table.insert(newcdefs,{stname=t.stname,funcname=t.funcname,args=args,argsc=argscsinpars,signature=t.signature.."nonUDT",cimguiname=t.cimguiname,call_args=call_args,ret =t.ret,comment=comment})
            --converting to Simple type----------------------------------------------------
            local defT3 = {}
            --first strings
            for k,v in pairs(defT) do
                defT3[k] = v
            end
            --then argsT table
            defT3.argsT = {}
            for k,v in ipairs(defT.argsT) do
                table.insert(defT3.argsT,{type=v.type,name=v.name})
            end
            local comma = (#defT.argsT > 0) and "," or ""
            --defT3.args = "("..defT.ret.." *pOut"..comma..defT.args:sub(2)
            defT3.ret = defT.ret.."_Simple"
            defT3.retorig = defT.ret
            defT3.ov_cimguiname = (defT3.ov_cimguiname or defT3.cimguiname).."_nonUDT2"
            defT3.nonUDT = 2
            defT3.retref = nil
            defsT[t.cimguiname][#defsT[t.cimguiname] + 1] = defT3
            defsT[t.cimguiname][t.signature.."nonUDT2"] = defT3
            table.insert(newcdefs,{stname=t.stname,funcname=t.funcname,args=args,argsc=argscsinpars,signature=t.signature.."nonUDT2",cimguiname=t.cimguiname,call_args=call_args,ret =t.ret,comment=comment})
        end
        end
		--end
    end
    for i,v in ipairs(newcdefs) do
        table.insert(FP.funcdefs,v)
    end
end

local function ADDdestructors(FP)
    local defsT = FP.defsT
    local newcdefs = {}
    --TODO add constructor = true
    for numcdef,t in ipairs(FP.funcdefs) do
        newcdefs[#newcdefs+1] = t
        if t.cimguiname then
            local defT = defsT[t.cimguiname]
            --local defT = cimf[t.signature]
        --for fname,defT in pairs(FP.defsT) do
        if not defT[1].ret and not defT[1].constructor then --if constructor not processed
            if defT[1].funcname:match("~") then
                defsT[t.cimguiname] = nil --clear destructor
                newcdefs[#newcdefs] = nil
            else
            for j,cons in ipairs(defT) do
                cons.constructor = true
            end
            assert(defT[1].stname==defT[1].funcname)
            local def = {}
            def.stname = defT[1].stname
            def.ret = "void"
            def.ov_cimguiname = def.stname.."_destroy"
            def.cimguiname = def.ov_cimguiname
            def.destructor = true
            def.args = "("..def.stname.."* self)"
            def.call_args = "(self)"
            def.signature = "("..def.stname.."*)"
            def.defaults = {}
            def.argsT = {{type=def.stname.."*",name="self"}}
            defsT[def.ov_cimguiname] = {def}
            defsT[def.ov_cimguiname][def.signature] = def
            newcdefs[#newcdefs+1]={stname=def.stname,funcname=def.ov_cimguiname,args=def.args,signature=def.signature,cimguiname=def.cimguiname,call_args=def.call_args,ret =def.ret}
            end
        end
        end
    end
    FP.funcdefs = newcdefs
end

--only basic ending
local c_types = {
    ["char"]=true,
    ["int"]=true,
    ["float"]=true,
    ["double"]=true,
    ["short"]=true,
    ["long"]=true,
    ["signed"]=true,
    ["unsigned"]=true,
    ["size_t"]=true,
    ["ptrdiff_t"]=true,
}
local function check_arg_detection(fdefs,typedefs)
    print"-----------------check arg detection---------------------------"
    for k,defT in pairs(fdefs) do
        for i,def in ipairs(defT) do
            for j,arg in ipairs(def.argsT) do
                --check name is not type, which happens in declaration without name
                if arg.name=="*" or not arg.type or not arg.name or c_types[arg.name] or typedefs[arg.name] then
                    print("bad argument name",arg.name, "in",def.funcname,def.args)
                end
            end
        end
    end
    print"-----------------end check arg detection-----------------------"
end
local function printItems(items)
	local function printItemsKind(items,kind)
		local res,resN = getRE()
		print("---------------------------------------",kind)
		local someit = items[kind]
		if someit then
			for i=1,#someit do print(someit[i]) end
		end
	end
	local res,resN = getRE()
	for k,v in pairs(resN) do
		printItemsKind(items,v)
	end
end
-------------
function M.Parser()
	local par = {}
	local cdefs ={}
	local itemsarr={}
	local items = {}
	par.itemsarr = itemsarr
	par.items = items
	par.defsT = {}
	par.funcdefs = {}
	par.embeded_structs = {}
	par.inerstructs = {}
	par.templates = {}
	par.typedefs_dict = {}
	par.cname_overloads = {}
	par.manuals = {}
	par.UDTs = {}

	function par:insert(line)
		table.insert(cdefs,line)
	end
	function par.getCname(stname,funcname)
		if #stname == 0 then return funcname end --top level
		local pre = stname.."_"
		return pre..funcname
	end
	function par.getCname_overload(stname,funcname,signature)
		local cname = par.getCname(stname,funcname)
		local ov_cname = par.cname_overloads[cname] and par.cname_overloads[cname][signature] --or cname
		return ov_cname
	end
	function par.get_manuals(def)
		return par.manuals[def.ov_cimguiname] or par.manuals[def.cimguiname]
	end
	function par:do_parse()
		self:parseItems()
		self:parseFunctions()
		self:compute_overloads()
	end
	function par:parseItems()
		--typedefs dictionary
		for i,line in ipairs(cdefs) do
			if line:match("typedef") then
				line = clean_spaces(line)
				local value,key = line:match("typedef%s+(.+)%s+([%w_]+);")
				if key and value then
					self.typedefs_dict[key] = value
				else --try function typedef
					local key = line:match("%(%*([%w_]+)%)%([^%(%)]*%)")
					if key then
						local linet = line
						linet = linet:gsub("typedef ","")
						linet = linet:gsub("%(%*("..key..")%)","(*)")
						self.typedefs_dict[key] = linet
					else
						print("not found function typedef")
						print(key,value,line)
					end
				end
			end
		end
		local txt = table.concat(cdefs,"\n")
		itemsarr,items = parseItems(txt)
		self.itemsarr , self.items = itemsarr,items
	end
	function par:printItems()
		printItems(items)
	end
	par.parseFunction = parseFunction

	function par:parseFunctions()
		for i,it in ipairs(itemsarr) do
			if it.re_name == "function_re" or it.re_name == "functionD_re" then
				self:parseFunction("",it.item)
			elseif it.re_name == "namespace_re" then
				local nsp = it.item:match("%b{}"):sub(2,-2)
				local namespace = it.item:match("namespace%s+(%S+)")
				local nspparr,itemsnsp = parseItems(nsp)
				for insp,itnsp in ipairs(nspparr) do
					if itnsp.re_name == "function_re" or itnsp.re_name == "functionD_re" then
						self:parseFunction("",itnsp.item,namespace)
					end
				end
			elseif it.re_name == "struct_re" then
				local nsp = it.item:match("%b{}"):sub(2,-2)
				local stname = it.item:match("struct%s+(%S+)")
				local nspparr,itemsnsp = parseItems(nsp)
				for insp,itnsp in ipairs(nspparr) do
					if itnsp.re_name == "function_re" or itnsp.re_name == "functionD_re" then
						self:parseFunction(stname,itnsp.item)
					elseif itnsp.re_name == "struct_re" then
						--get embeded_structs
						local embededst = itnsp.item:match("struct%s+(%S+)")
						self.embeded_structs[embededst] = stname.."::"..embededst
						local nsp2 = strip_end(itnsp.item:match("%b{}"):sub(2,-2))
						local itemsemarr,itemsem = parseItems(nsp2)
						assert(not itemsem.struct_re,"two level embed struct")
						for iemb,itemb in ipairs(itemsemarr) do
							if itemb.re_name == "function_re" or itemb.re_name == "functionD_re" then
								self:parseFunction(embededst,itemb.item)
							end
						end
					end
				end
			end
		end
		--require"anima"
		--prtable(self.defsT)
	end
	function par:clean_struct(stru)
		local outtab = {}
		local iner = strip_end(stru:match("%b{}"):sub(2,-2))
		local stname = stru:match("struct%s*(%S+)%s*%b{}")
		if not stname then
			print(stru)
			error"could not get stname"
		end
		--initial
		table.insert(outtab,stru:match("(.-)%b{}"))
		table.insert(outtab,"{")
		local itlist,itemsin = parseItems(iner)
		if #itlist == 0 then return "" end --here we avoid empty structs
		for j,it in ipairs(itlist) do
			if it.re_name == "vardef_re" or it.re_name == "functype_re" or it.re_name == "union_re" then
				local it2 = it.item --:gsub("<([%w_]+)>","_%1") --templates
				--local ttype,template = it.item:match("([^%s,%(%)]+)%s*<(.+)>")
				local ttype,template =    it.item:match"([^%s,%(%)]+)%s*<(.+)>"
				if template then
					local te = template:gsub("%s","_")
					te = te:gsub("%*","Ptr")
					self.templates[ttype] = self.templates[ttype] or {}
					self.templates[ttype][template] = te
					it2 = it2:gsub("<([%w_%*%s]+)>","_"..te)
				end
				--clean mutable
				it2 = it2:gsub("mutable","")
				--skip static variables
				if not (it.re_name == "vardef_re" and it2:match"static") then
					table.insert(outtab,it2)
				end
			elseif it.re_name == "struct_re" then
				table.insert(self.inerstructs,it)
			end
		end
		--final
		table.insert(outtab,"\n};")
		return table.concat(outtab,""),stname,outtab
	end
	function par:gen_structs_and_enums()
		local outtab = {}
		local outtabpre = {}
		local typedefs_table = {}

		--first typedefs
		for i,it in ipairs(itemsarr) do
			if it.re_name == "typedef_re" or it.re_name == "functypedef_re" or it.re_name == "vardef_re" then
				table.insert(outtabpre,it.item)
				-- add typedef after struct name
				if it.re_name == "vardef_re" and it.item:match"struct" then
					local stname = it.item:match("struct%s*(%S+)%s*;")
					table.insert(typedefs_table,"typedef struct "..stname.." "..stname..";\n")
					self.typedefs_dict[stname]="struct "..stname
				end
			end
		end
		--then structs and enums
		for i,it in ipairs(itemsarr) do
			if it.re_name == "enum_re" then
				table.insert(outtab,it.item)
			elseif it.re_name == "struct_re" then
				local cleanst,structname = self:clean_struct(it.item)
				if structname then
				table.insert(outtab,cleanst)
				table.insert(typedefs_table,"typedef struct "..structname.." "..structname..";\n")
				self.typedefs_dict[structname]="struct "..structname
				end
			end
		end
		--inner_structs
		for i,it in ipairs(self.inerstructs) do
			local cleanst,structname = self:clean_struct(it.item)
			if structname then
			table.insert(outtab,cleanst)
			table.insert(typedefs_table,"typedef struct "..structname.." "..structname..";\n")
			self.typedefs_dict[structname]="struct "..structname
			end
		end
		local uniques = {}
		for i,l in ipairs(typedefs_table) do
			if not uniques[l] then
				uniques[l] = true
				table.insert(outtabpre,1,l)
			end
		end
		--check arg detection failure if no name in function declaration
		check_arg_detection(self.defsT,self.typedefs_dict)
		return table.concat(outtabpre,""),table.concat(outtab,"")
	end
	-----------
	function par:parse_struct_line(line,outtab)
		local functype_re = "^%s*[%w%s%*]+%(%*[%w_]+%)%([^%(%)]*%)"
		local functype_reex = "^(%s*[%w%s%*]+%(%*)([%w_]+)(%)%([^%(%)]*%))"
		line = clean_spaces(line)
		if line:match(functype_re) then
			local t1,name,t2 = line:match(functype_reex)
			table.insert(outtab,{type=t1..t2,name=name})
		else
			--split type name1,name2; in several lines
			local typen,rest = line:match("%s*([^,]+)%s(%S+[,;])")
			--local template_type = typen:match("/%*<(.+)>%*/")
			--if template_type then typen = typen:match("(.+)/%*") end
                if not typen then -- Lets try Type*name
                    typen,rest = line:match("([^,]+%*)(%S+[,;])")
                end
			local template_type 
			for k,v in pairs(self.templates) do
				template_type = typen:match(k.."_(.+)")
				if template_type then break end
			end
			if template_type then 
				template_type = template_type:gsub("_"," ")
				template_type = template_type:gsub("Ptr","%*")
			end
			for name in rest:gmatch("([^%s,;]+)%s?[,;]") do
				table.insert(outtab,{type=typen,template_type=template_type,name=name})
			end
		end
	end
	function par:gen_structs_and_enums_table()
		local outtab = {enums={},structs={}}
		local outtabpre = {}
		local typedefs_table = {}

		self.inerstructs = {}
		--then structs and enums
		for i,it in ipairs(itemsarr) do
			if it.re_name == "enum_re" then
				local enumname = it.item:match"^%s*enum%s+([^%s;{}]+)"
				outtab.enums[enumname] = {}
				local inner = strip_end(it.item:match("%b{}"):sub(2,-2))
				local enumarr = str_split(inner,",")
				for j,line in ipairs(enumarr) do
					local name,value = line:match("%s*([%w_]+)%s*=%s*([^,]+)")
					if value then
						table.insert(outtab.enums[enumname],{name=name,value=value})
					else --increment by one
						local name = line:match("%s*([^,]+)")
						local enum_table = outtab.enums[enumname]
						local value = enum_table[#enum_table] and (enum_table[#enum_table].value + 1) or 0
						table.insert(outtab.enums[enumname],{name=name,value=value})
					end
				end
			elseif it.re_name == "struct_re" then
				local cleanst,structname,strtab = self:clean_struct(it.item)
				if structname then --not empty struc
					outtab.structs[structname] = {}
					for j=3,#strtab-1 do
						self:parse_struct_line(strtab[j],outtab.structs[structname])
					end
				end
			end
		end
		--inner_structs

		for i,it in ipairs(self.inerstructs) do
			local cleanst,structname,strtab = self:clean_struct(it.item)
			if structname then --not empty struc
				outtab.structs[structname] = {}
				for j=3,#strtab-1 do
					self:parse_struct_line(strtab[j],outtab.structs[structname])
				end
			end
		end
		--[[
		local uniques = {}
		for i,l in ipairs(typedefs_table) do
			if not uniques[l] then
				uniques[l] = true
				table.insert(outtabpre,1,l)
			end
		end
		--]]
		--calcule size of name[16+1] [xxx_COUNT]
		local allenums = {}
		--first calc_value in enums
		for enumname,enum in pairs(outtab.enums) do
			for i,t in ipairs(enum) do
				local val = tonumber(t.value)
				if val then
					t.calc_value = val
				elseif t.value:match"<<" then
					local v1,v2 = t.value:match("(%d+)%s*<<%s*(%d+)")
					t.calc_value = bit.lshift(v1,v2)
				elseif t.value:match"|" then --or several enums
					local ens = t.value
					ens = strsplit(ens,"|")
					for i,v in ipairs(ens) do ens[i] = allenums[clean_spaces(v)] end
					t.calc_value = bit.bor(unpack(ens))
				elseif allenums[t.value] then
					t.calc_value = allenums[t.value]
				else
					print("Error unknown value in enums",t.value)
				end
				assert(t.calc_value)
				allenums[t.name] = t.calc_value
			end
		end
		    --then calcsize in struct members
		for stname,struct in pairs(outtab.structs) do
			for i,t in ipairs(struct) do
				local val = t.name:match"%[([^%[%]]+)%]"
				if val then
					if tonumber(val) then
						t.size = tonumber(val)
					elseif allenums[val] then
						t.size = allenums[val]
					elseif val:match"%+" then
						local s1,s2 = val:match("(%d+)%s*%+%s*(%d+)")
						t.size = s1+s2
					else
						print("Error size is",val)
					end
					assert(t.size)
				end
			end
		end
		return outtab
	end
	par.alltypes = {}
    local function get_types(v)
        for i,t in ipairs(v) do
            local signature = t.signature:sub(2,-2) -- without parenthesis
            for typec in signature:gsub("(%(.-%))", function(x) return x:gsub(",","\0") end):gmatch(".-([^,]+),?") do
                local key = typec:gsub("%z+", ",")
                par.alltypes[key] = true
            end
        end
    end
    function par:dump_alltypes()
        for k,v in pairs(self.alltypes) do print(k, typetoStr(k) ) end
    end
    function par:compute_overloads()
        local strt = {}
        local numoverloaded = 0
        self.alltypes = {}
        table.insert(strt,"----------------overloadings---------------------------")
        --require"anima.utils" 
        for k,v in pairs(self.defsT) do
            get_types(v)
            if #v > 1 then
                numoverloaded = numoverloaded + #v
                --print(k,#v)
                table.insert(strt,string.format("%s\t%d",k,#v))
                local typesc,post = name_overloadsAlgo(v)
                for i,t in ipairs(v) do
                    --take overloaded name from manual table or algorythm
                    t.ov_cimguiname = self.getCname_overload(t.stname,t.funcname,t.signature) or k..typetoStr(post[i])
                    table.insert(strt,string.format("%d\t%s\t%s %s",i,t.ret,t.ov_cimguiname,t.signature))
                    --prtable(typesc[i])
                end
                --check not two names are equal (produced by bad cimguiname_overload)
                for i=1,#v-1 do 
                    for j=i+1,#v-1 do
                        if v[i].ov_cimguiname == v[j].ov_cimguiname then
                          local t,tj = v[i],v[j]
                          print("Error caused by Bad overloading "..t.ov_cimguiname.." of function ",t.funcname,t.signature,"conflicts with ",tj.funcname,tj.signature)
                          error("Bad overloading:"..t.ov_cimguiname) 
                        end
                    end
                end
			else
				v[1].ov_cimguiname = v[1].cimguiname
            end
        end
        --print(numoverloaded, "overloaded")
        table.insert(strt,string.format("%d overloaded",numoverloaded))
		AdjustArguments(self)
		ADDnonUDT(self)
		ADDdestructors(self)
        self.overloadstxt  = table.concat(strt,"\n")
    end
	return par
end

------serializeTable("anyname",table) gives a string that recreates the table with dofile(generated_string)
local function serializeTable(name, value, saved)
    
    local function basicSerialize (o)
        if type(o) == "number" or type(o)=="boolean" then
            return tostring(o)
        elseif type(o) == "string" then
            return string.format("%q", o)
        else
            return "nil"
        end
    end
    
    local string_table = {}
    if not saved then 
        table.insert(string_table, "local "..name.." = ") 
    else
        table.insert(string_table, name.." = ") 
    end
    
    saved = saved or {}       -- initial value
    
    if type(value) == "number" or type(value) == "string" or type(value)=="boolean" then
        table.insert(string_table,basicSerialize(value).."\n")
    elseif type(value) == "table" then
        if saved[value] then    -- value already saved?
            table.insert(string_table,saved[value].."\n")          
        else
            saved[value] = name   -- save name for next time
            table.insert(string_table, "{}\n")  
---[[
            local ordered_keys = {}
            for k,v in pairs(value) do
                table.insert(ordered_keys,k)
            end
            local function sorter(a,b)
                if type(a)==type(b) then 
                    return a<b 
                elseif type(a)=="number" then
                    return true
                else
                    assert(type(b)=="number")
                    return false
                end
            end
            table.sort(ordered_keys,sorter)
            for _,k in ipairs(ordered_keys) do
                local v = value[k]
--]]
           -- for k,v in pairs(value) do      -- save its fields

                local fieldname = string.format("%s[%s]", name,basicSerialize(k))
                table.insert(string_table, serializeTable(fieldname, v, saved))
            end
        end
    --else
        --error("cannot save a " .. type(value))
    end
    
    return table.concat(string_table)
end
M.serializeTable = serializeTable
M.serializeTableF = function(t)
	return M.serializeTable("defs",t).."\nreturn defs"
end
--iterates lines from a gcc/clang -E in a specific location
local function location(file,locpathT,defines)
	local define_re = "^#define%s+([^%s]+)%s+([^%s]+)$"
	local number_re = "^-?[0-9]+u*$"
	local hex_re = "0x[0-9a-fA-F]+u*$"
    local location_re 
    if COMPILER == "cl" then
        location_re = '^#line (%d+) "([^"]*)"'
    else --gcc, clang
        location_re = '^# (%d+) "([^"]*)"'
    end
    local path_reT = {}
    for i,locpath in ipairs(locpathT) do
        table.insert(path_reT,'^.*[\\/]('..locpath..')%.h$')
    end
    local in_location = false
    local which_location = ""
    local loc_num
    local loc_num_incr
    local lineold = "" 
    local which_locationold,loc_num_realold
    local lastdumped = false
    local function location_it()
        repeat
            local line = file:read"*l"
            if not line then
                if not lastdumped then
                    lastdumped = true
                    return lineold, which_locationold,loc_num_realold
                else
                    return nil
                end
            end
            if #line==0 then --nothing on emptyline
            elseif not line:match("%S") then --nothing if only spaces
            elseif line:sub(1,1) == "#" then
                -- Is this a location pragma?
                local loc_num_t,location_match = line:match(location_re)
                if location_match then
                    in_location = false
                    for i,path_re in ipairs(path_reT) do
						local locpath = location_match:match(path_re)
                        if locpath then 
                            in_location = true;
                            loc_num = loc_num_t
                            loc_num_incr = 0
                            which_location = locpath --locpathT[i]
                            break 
                        end
                    end
				elseif in_location then
					local name,val = line:match(define_re)
					if name and val then
						--while defines[val] do val = defines[val] end
						--if val:match(number_re) or val:match(hex_re) then
							table.insert(defines,{name , val})
						--end
					end
                end
				
            elseif in_location then
                local loc_num_real = loc_num + loc_num_incr
                loc_num_incr = loc_num_incr + 1
				--if doprint then print(which_locationold,which_location) end
                if (which_locationold~=which_location) or (loc_num_realold and loc_num_realold < loc_num_real) then
                    --old line complete
					--doprint = false
                    local lineR,which_locationR,loc_num_realR = lineold, which_locationold,loc_num_realold
                    lineold, which_locationold,loc_num_realold = line,which_location,loc_num_real
                    return lineR,which_locationR,loc_num_realR
                else
                    lineold=lineold..line
                    which_locationold,loc_num_realold = which_location,loc_num_real
                --return line,loc_num_real, which_location
                end
            end
        until false --forever
    end
    return location_it
end
M.location = location
return M