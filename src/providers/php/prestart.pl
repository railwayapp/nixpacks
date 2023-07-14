#!/usr/bin/env perl

undef $/;

use strict;
use warnings;

use FindBin;
use lib ("$FindBin::RealBin");

use File::Find;
use Nixpacks::Config::Template qw(compile_template);
use Nixpacks::Util::Logger;
use Nixpacks::Util::ChmodRecursive qw(chmod_recursive);
use Nixpacks::Util::Laravel qw(is_laravel check_possible_env_errors);

my $server_logger = Nixpacks::Util::Logger->new("server");

Nixpacks::Util::ChmodRecursive::chmod_recursive("/app/storage") if -e "/app/storage";

if ($#ARGV != 1) {
    print STDERR "Usage: $0 <config-file> <output-file>\n";
    exit 1;
}

if(Nixpacks::Util::Laravel::is_laravel()) {
    Nixpacks::Util::Laravel::check_possible_env_errors("/app");
}

Nixpacks::Config::Template::compile_template($ARGV[0], $ARGV[1]);
my $port = $ENV{"PORT"};
$server_logger->info("Server starting on port $port");