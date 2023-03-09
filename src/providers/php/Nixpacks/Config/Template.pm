package Nixpacks::Config::Template;

use Nixpacks::Nix;

sub if_stmt {
    my ($condition, $value, $else) = @_;

    if($ENV{$condition} ne "") {
        return $value;
    } else {
        return $else;
    }
}

sub compile_template {
    my ($infile, $outfile) = @_;
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
        
        # Nix paths
        s/\$\!\{(\w+)\}/Nixpacks::Nix::get_nix_path($1)/eg;

        $out .= $_;
    }
    close(FH);
    open(FH, '>', $outfile) or die "Could not write configuration file '$outfile' $!";
    print FH $out;
    close(FH);
}

1;