package Nixpacks::Util::Laravel;

use File::Spec::Functions qw(catfile);
use Nixpacks::Util::Logger;

my %variable_hints = (
    APP_ENV => 'You should probably set it to `production`.',
);

my $logger = Nixpacks::Util::Logger->new("laravel");

sub is_laravel {
    $ENV{IS_LARAVEL} ne "";
}

sub check_variable {
    my ($varname) = @_;

    if($ENV{$varname} eq "") {
        my $hint = "Your app configuration references the $varname environment variable, but it is not set.";
        if(defined $variable_hints{$varname}) {
            $hint .= ' ' . $variable_hints{$varname};
        }
        $logger->warn($hint);
    }
}

sub check_possible_env_errors {
    my ($srcdir) = @_;

    my $config_path = catfile($srcdir, 'config', '*.php');
    my @config_files = glob($config_path);

    foreach my $config_file (@config_files) {
        open(FH, '<', $config_file);

        while(<FH>) {
            check_variable($1) if /env\(["']([^,]*)["']\)/ and $1 ne "APP_KEY";
        }
    }
	
	if($ENV{APP_KEY} eq "") {
		$logger->warn("Your app key is not set! Please set a random 32-character string in your APP_KEY environment variable. This can be easily generated with `openssl rand -hex 16`.")
	}
}

1;