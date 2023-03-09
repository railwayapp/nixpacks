<!DOCTYPE html>
<html>
	<head>
		<title>Hello World!</title>
		<style>
			@import url('https://fonts.bunny.net/css2?family=Nunito&display=swap');
			body {
			  background-color: rgb(245, 234, 214);
			  font-family: Nunito;
			  font-size: large;
			  display: flex;
			  align-items: center;
			  justify-content: center;
			  height: 100vh;
			  text-align: center;
			  padding: 0;
			  margin: 0;
			}

			a {
			  color: #dd5500;
			  text-decoration: none;
			  transition-property: all;
			  transition-duration: .2s;
			  padding-bottom: 0px;
			  border-bottom: 0px solid transparent;
			}
			a:hover {
			  border-bottom-width: 2px;
			  border-bottom-color: #dd5500;
			}
		</style>
	</head>
	<body>
		<div>
		  <h1>Hello World!</h1>
		  <p>Welcome to <a href="https://github.com/railwayapp/nixpacks">Nixpacks</a>!</p>
		  <p><b>PHP Version:</b> <?php echo phpversion() ?></p>
		</div>
	</body>
</html>