#!/usr/bin/env perl

undef $/;

sub if_stmt {
    my $condition = $_[0];
    my $value = $_[1];
    my $else = $_[2];

    if($ENV{$condition} eq "yes") {
        return $value;
    } else {
        return $else;
    }
}

if ($#ARGV != 1) {
    print STDERR "Usage: $0 <config-file> <output-file>\n";
    exit 1;
}
my $infile = $ARGV[0];
my $outfile = $ARGV[1];
open(FH, '<', $infile) or die "Could not open configuration file '$infile' $!";
my $out = '';
while (<FH>) {

    # If statements
    s{
        \$if\s*\((\w+)\)\s*\(
            ([\s\S]*?)
        \)\s*else\s*\(
            ([\s\S]*?)
        \)
    }{if_stmt($1, $2, $3)}egx;

    # Variables
    s/\$\{(\w+)\}/$ENV{$1}/eg;

    $out .= $_;
}
close(FH);
open(FH, '>', $outfile) or die "Could not write configuration file '$outfile' $!";
print FH $out;
close(FH);