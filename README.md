drop
====

A simple screenshot tool with S3 support written in rust.


Dependencies
------------

* slop - https://github.com/naelstrof/slop
* imagemagick - https://www.imagemagick.org
* s3cmd - http://s3tools.org/s3cmd
* xsel - http://www.vergenet.net/%7Econrad/software/xsel/


Installation
------------

Clone this repo

    git clone git@github.com:gilbertw1/drop.git

Place ```config.toml.default``` in ```~/.config/drop/config.toml``` and edit it with required values

    mkdir -p ~/.config/drop
    cp config.toml.default ~/.config/drop/config.toml
    
Build drop:

    cargo build --release
  
Run drop:

    ./target/release/drop

Usage
-----

### Take a screenshot

Drop can be used to take a screenshot and allows you to select a portion of the screen to save and upload. Additionally, a single window can be screenshotted by simply clicking on the window. Once the screenshot has been saved and upload a notification will popup saying the screenshot has been upload and a url to the screenshot will be saved in the clipboard.

    drop
    

### Take a screenshot
   
Drop can be used to upload a file to S3, resulting in a url to the uploaded file added to the system clipboard.

    drop -f <file>


### Help

Help can be accessed easily by using the ```-h``` flag.

    drop -h

Configuration
-------------

The drop configuration file should be placed at ```~/.config/drop/config.toml```. It's values are as follows:

```toml
    [drop]
    dir = '~/.drop'   # Directory used to save screenshots (DEFAULT: ~/.drop)
    host = ''    # Custom domain used to generate screenshot links (DEFAULT: empty)
    unique_length = 10    # Length of unique string used in creating filenames (DEFAULT: 10)
    filename_strategy = APPEND # Naming strategy to use when uploading file (DEFAULT: APPEND)
                               #   VALUES:           
                               #       APPEND: Append unique string to filename
                               #       EXACT: Don't alter filename when uploading
                               #       REPLACE: Replace filename with unique string

    [aws]
    bucket = 'drop'    # S3 bucket to upload screenshots & files to (DEFAULT: empty)
    key = ''     # AWS credentials used to authenticate with S3 (DEFAULT: empty)
    secret = ''     # AWS secret used to authenticate with S3 (DEFAULT: empty)
```

* More info on aws access keys [here](https://aws.amazon.com/developers/access-keys/)
* If bucket, key, or secret is missing drop will only save the screenshot locally

S3 Setup
--------

If configuring Drop to upload to S3 a bucket will be required. Info on creating S3 buckets can be found here: http://docs.aws.amazon.com/AmazonS3/latest/dev/UsingBucket.html. Once a bucket has been created, if you want the drops to be publicly accessible the following policy should be added to the bucket:

    {
      "Version": "2008-10-17",
      "Statement": [{
        "Sid": "AllowPublicRead",
        "Effect": "Allow",
        "Principal": { "AWS": "*" },
        "Action": ["s3:GetObject"],
        "Resource": ["arn:aws:s3:::BUCKET_NAME/*" ]
      }]
    }
    
* Replace ```bucket``` with the name of your bucket.


Custom Domain Setup
-------------------

In order for Drop to work correctly with a custom domain, a few additional steps need to be taken.

* Create an S3 bucket with the name of the custom domain. For example if I want my drops available at ```drop.mydomain.com```, then the S3 bucket I create needs to be named ```drop.mydomain.com```.

* Enable static website hosting for the bucket. This can be done by selecting the bucket and checking "Enable website hosting" under the "Static Website Hosting" section in the bucket properties. Note that you will need at least a blank html file in the bucket to select as the required 'Index Document'.

* Add a cname record to your DNS pointing to the static S3 website endpoint (this can be found in the "Static Website Hosting" properties section). In the above example, in the DNS settings for ```mydomain.com```, a cname entry for ```drop``` would be added pointing to the static S3 site.

* Update the ```~/.config/drop/config.toml``` file with the variable ```host``` pointing to your custom domain (```drop.mydomain.com```)
