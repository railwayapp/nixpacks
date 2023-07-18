package Nixpacks::Config::Template;

require 5.013002;

use Nixpacks::Nix;

sub if_stmt {
    my ($condition, $value, $else) = @_;

    if($ENV{$condition} ne "") {
        return replace_str($value);
    } else {
        return replace_str($else);
    }
}

sub replace_str {
    my ($input) = @_;
    my $new = 
        $input 
        =~ 
            # If statements
            s{
                \$if\s*\((\w+)\)\s*\(
                    ([\s\S]*?)
                \)\s*else\s*\(
                    ([\s\S]*?)
                \)
            }{if_stmt($1, $2, $3)}egxr
        =~
            # Variables
            s/\$\{(\w+)\}/$ENV{$1}/egr
        =~ 
            # Nix paths
            s/\$\!\{(\w+)\}/Nixpacks::Nix::get_nix_path($1)/egr;
    return $new;
}

sub compile_template {
    my ($infile, $outfile) = @_;
    open(my $handle, '<', $infile) or die "Could not open configuration file '$infile' $!";
    my $out = '';
    while (my $line = <$handle>) {
        $out .= replace_str($line);
    }
    close(FH);
    open(FH, '>', $outfile) or die "Could not write configuration file '$outfile' $!";
    print FH $out;
    close(FH);
}

1;