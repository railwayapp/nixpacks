#!/usr/bin/env perl
use strict;
use warnings;

if ($#ARGV != 1) {
    print STDERR "Usage: $0 <config-file> <output-file>\n";
    exit 1;
}
my $infile = $ARGV[0];
my $outfile = $ARGV[1];
open(FH, '<', $infile) or die "Could not open configuration file '$infile' $!";
my $out = '';
while (<FH>) {
    s/\$\{(\w+)\}/$ENV{$1}/eg;
    $out .= $_;
}
close(FH);
open(FH, '>', $outfile) or die "Could not write configuration file '$outfile' $!";
print FH $out;
close(FH);