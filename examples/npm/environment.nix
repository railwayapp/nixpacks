{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell { 
  buildInputs = [ pkgs.stdenv pkgs.nodejs ]; 
}
