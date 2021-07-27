#!/usr/bin/env python

import requests, time

serv = 'http://localhost:8000'

def new_session() :
    s = requests.session()
    s.headers.update({'Content-Type': 'application/json'})
    return s

def login(s, user, pw, scopes) :
    req = {
        'name': user,
        'secret': pw,
        'scopes': scopes,
    }
    v = s.post(serv + '/auth', json=req).json()
    if v['status'] == 'ok' :
        tok = v['result']['token']
        s.headers.update({'Authorization': 'bearer ' + tok})
    return v

def check(s) :
    return s.get(serv + '/auth').json()

def create_user(s, user, pw, life, scopes) :
    req = {
        "name": user,
        "secret": pw,
        "life": life,
        "scopes": scopes,
    }
    return s.post(serv + '/admin/user', json=req).json()

def create_scope(s, scope) :
    return s.post(serv + '/admin/scope', json=scope).json()

def clean(s) :
    return s.post(serv + "/admin/clean").json()

s = new_session()

if 1 :
    print login(s, 'admin', 'adminadmin', ['authadmin'])
    if 1 :
        print check(s)

    if 0 :
        print create_scope(s, "user")

    if 0 :
        print create_user(s, "test", "testpw", 60*60*365*5, ["user"])

    if 1 :
        print clean(s)

if 0 :
    s = new_session()
    print login(s, "test", "testpw", ["user"])


