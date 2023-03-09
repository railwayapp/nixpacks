#!/usr/bin/env perl

undef $/;

use FindBin;
use lib ("$FindBin::RealBin");

use File::Find;
use Nixpacks::Config::Template;
use Nixpacks::Util::ChmodRecursive;


Nixpacks::Util::ChmodRecursive::chmod_recursive("/app/storage") if -e "/app/storage";

if ($#ARGV != 1) {
    print STDERR "Usage: $0 <config-file> <output-file>\n";
    exit 1;
}

Nixpacks::Config::Template::compile_template($ARGV[0], $ARGV[1]);
my $port = $ENV{"PORT"};
print "Server starting on port $port\n";