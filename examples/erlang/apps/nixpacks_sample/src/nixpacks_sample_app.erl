%%%-------------------------------------------------------------------
%% @doc nixpacks_sample public API
%% @end
%%%-------------------------------------------------------------------

-module(nixpacks_sample_app).

-behaviour(application).

-export([start/2, stop/1]).

start(_StartType, _StartArgs) ->
    io:format("Hello from Erlang~n"),
    erlang:halt(0).

stop(_State) ->
    ok.

%% internal functions
