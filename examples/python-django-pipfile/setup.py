from setuptools import setup, find_packages

setup(
    name='mypackage',
    version='0.0.1',
    packages=find_packages(),
    install_requires=[
        'lxml >= 3.3.5'
    ],
    zip_safe=False
)