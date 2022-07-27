%%%-------------------------------------------------------------------
%% @doc nixpacks_sample public API
%% @end
%%%-------------------------------------------------------------------

-module(nixpacks_sample_app).

-behaviour(application).

-export([start/2, stop/1]).

start(_StartType, _StartArgs) ->
    nixpacks_sample_sup:start_link().

stop(_State) ->
    ok.

%% internal functions
