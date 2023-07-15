<?php
namespace App\Controllers;

class Greeting {
	public static function greet(string $name) {
		header('Content-Type: text/plain');
		echo "Hello, $name!";
	}
}