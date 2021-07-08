drop
====

A simple screenshot, screencast, and file upload tool with S3 support written in rust.


Dependencies
------------

### Linux

*Required*

* s3cmd - http://s3tools.org/s3cmd
* xsel - http://www.vergenet.net/%7Econrad/software/xsel/

*Optional (X11)*

* slop (screenshot + screencast) - https://github.com/naelstrof/slop
* imagemagick (screenshot + gif screencast) - https://www.imagemagick.org
* ffmpeg (screencast) - https://ffmpeg.org

*Optional (Wayland)*

* slurp (screenshot + screencast) - https://github.com/emersion/slurp
* grim (screenshot) - https://github.com/emersion/grim
* wf-recorder (screencast) - https://github.com/ammen99/wf-recorder
* imagemagick (gif screencast) - https://www.imagemagick.org

### MacOS

*Required*

* s3cmd - http://s3tools.org/s3cmd


Installation
------------

### Source

Clone this repo

    git clone git@github.com:gilbertw1/drop.git

Optionally place ```config.toml.default``` in ```~/.config/drop/config.toml``` and edit it with
desired values

    mkdir -p ~/.config/drop
    cp config.toml.default ~/.config/drop/config.toml
    
Build drop:

    cargo build --release
  
Run drop:

    ./target/release/drop
    
    
### Arch Linux

AUR - https://aur.archlinux.org/packages/drop

    yaourt -S drop

### MacOS

HomeBrew - https://github.com/gilbertw1/homebrew-drop

    brew install gilbertw1/drop/drop

Usage
-----

### Help

Comprehensive help can be accessed easily by using the ```-h``` flag.

    drop -h
    

### Take a screenshot

Drop can be used to take a screenshot and allows you to select a portion of the screen to save and
upload. Additionally, a single window can be screenshotted by simply clicking on the window. Once
the screenshot has been saved and upload a notification will popup saying the screenshot has been
uploaded and a url to the screenshot will be saved in the clipboard.

    drop -s
    

### Take a screencast

Drop can be used to take a screencast. This behaves identically to taking a screenshot except it
records a video. A status bar icon will appear that can be used to terminate the screencast.

    drop -v

Optionally include audio (off by default) 

    drop -v -a
    
Create screencast as a gif (Linux Only)

    drop -v --video-format gif
    

### Upload file
   
Drop can be used to upload a file to S3, resulting in a url to the uploaded file added to the system
clipboard.

    drop <file>

By default drop will apply a randomly generated string to the filename, however this behavior can be
overridden

    drop --filename-strategy exact <file>


### Create and upload file from stdin

Drop can be used to create and upload a file from stdin when pass '-' as a filename

    echo "Some Text" | drop -

You can specify a extension to be applied to the created file

    curl http://api.bryangilbert.com/profile | drop -e json -

You can also specify a full filename to be used:

    echo "<html><h1>Hello</h1></html>" | drop -f index.html --filename-strategy exact -

Configuration
-------------

The drop configuration file should be placed at ```~/.config/drop/config.toml```. If one doesn't
exist, it will be created the first time drop is run. It's values are as follows:

```toml
    [drop]
    dir = '~/.drop'                 # Directory used to save generated files (DEFAULT: ~/.drop)
    host = ''                       # Custom domain used to generate screenshot links (DEFAULT: empty)
    unique_length = 10              # Length of unique string used in creating filenames (DEFAULT: 10)
    filename_strategy = 'prepend'   # Naming strategy to use when uploading file (DEFAULT: PREPEND)
                                    #   VALUES:           
                                    #       append: Append unique string to filename
                                    #       prepend: Prepend unique string to filename
                                    #       exact: Don't alter filename when uploading
                                    #       replace: Replace filename with unique string
    audio_source = 'mic'            # Specifies audio source to use when recording audio during a
                                    # screen cast. Possible values are mic, desktop (Linux only)
                                    # (default: mic)
    [aws]
    bucket = 'drop'                 # S3 bucket to upload screenshots & files to (DEFAULT: empty)
    key = ''                        # AWS credentials used to authenticate with S3 (DEFAULT: empty)
    secret = ''                     # AWS secret used to authenticate with S3 (DEFAULT: empty)
```

* More info on aws access keys [here](https://aws.amazon.com/developers/access-keys/)
* If aws bucket, key, or secret is missing drop will only save the screenshot locally

Reference
---------

```
drop 0.3.3                                                                                                                             │
Bryan G. <bryan@bryan.sh>                                                                                                              │
Screenshot & file upload tool with S3 support - http://github.com/gilbertw1/drop                                                       │
                                                                                                                                       │
USAGE:                                                                                                                                 │
    drop [FLAGS] [OPTIONS] [FILE]                                                                                                      │
                                                                                                                                       │
FLAGS:                                                                                                                                 │
    -a, --audio           Enable audio in screencast                                                                                   │
    -b, --border          Display border around screencast area while recording (does not show in video, Linux only)                   │
    -h, --help            Prints help information                                                                                      │
    -l, --local           Don't upload file to remote location (produces local file url)                                               │
    -m, --mouse           Show mouse cursor in screencast.                                                                             │
    -i, --no-tray-icon    Do not display tray icon while recording screencast.                                                         │
    -q, --quiet           Hide desktop notifications                                                                                   │
    -s, --screenshot      Capture screenshot                                                                                           │
    -t, --transparent     Enable transparent selection overlay, compositor is required (Linux only)                                    │
    -V, --version         Prints version information                                                                                   │
        --verbose         Enables verbose logging                                                                                      │
    -v, --video           Record video screencast                                                                                      │
                                                                                                                                       │
OPTIONS:                                                                                                                               │
        --audio-source <AUDIO_SOURCE>        Audio source to use when creating screencast. (desktop only available on                  │
                                             Linux) [default: mic]  [possible values: mic, desktop]                                    │
        --aws-bucket <AWS_BUCKET>            S3 Bucket to upload to                                                                    │
        --aws-key <AWS_KEY>                  AWS access key                                                                            │
        --aws-secret <AWS_SECRET>            AWS access secret                                                                         │
    -d, --delay <SECONDS>                    Number of seconds to delay screenshot or screencast start. [default: 0]                   │
        --display-server <DISPLAY_SERVER>    Indicates which display server to target (Linux only - Defaults to                        │
                                             $XDG_SESSION_TYPE or x11) [possible values: x11, wayland]                                 │
    -e, --extension <EXTENSION>              Extension to use when creating a filename                                                 │
    -f, --filename <FILENAME>                Filename to use for creating resulting file                                               │
        --filename-strategy <STRATEGY>       File upload naming strategy [possible values: exact, append, prepend,                     │
                                             replace]                                                                                  │
        --host <HOST>                        Custom host                                                                               │
    -u, --unique-length <LENGTH>             Length of unique string used to create filenames                                          │
        --video-format <FORMAT>              Format to record screencast [default: mp4]  [possible values: mp4, gif]                   │
                                                                                                                                       │
ARGS:                                                                                                                                  │
    <FILE>    Optional file to upload. If equal to '-' then drop reads from stdin
```

S3 Setup
--------

If configuring Drop to upload to S3, a bucket will be required. Info on creating S3 buckets can be
found here: http://docs.aws.amazon.com/AmazonS3/latest/dev/UsingBucket.html. Once a bucket has been
created, if you want the drops to be publicly accessible the following policy should be added to the
bucket:

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
    
* Replace ```bucket``` in the config file with the name of your bucket.


Custom Domain Setup
-------------------

In order for Drop to work correctly with a custom domain, a few additional steps need to be taken.

* Create an S3 bucket with the name of the custom domain. For example if I want my drops available
at ```drop.mydomain.com```, then the S3 bucket I create needs to be named ```drop.mydomain.com```.

* Enable static website hosting for the bucket. This can be done by selecting the bucket and
checking "Enable website hosting" under the "Static Website Hosting" section in the bucket
properties. Note that you will need at least a blank html file in the bucket to select as the
required 'Index Document'. A simple blank index document can be created and uploaded using the
following drop command: 

```bash
    echo "<html></html>" | drop -f index.html --filename-strategy exact -
```

* Add a cname record to your DNS pointing to the static S3 website endpoint (this can be found in
the "Static Website Hosting" properties section). In the above example, in the DNS settings for
```mydomain.com```, a cname entry for ```drop``` would be added pointing to the static S3 site.

* Update the ```~/.config/drop/config.toml``` file with the variable ```host``` pointing to your
custom domain (```drop.mydomain.com```)


Roadmap
-------
* Better error handling
* Add support for selecting portion of screen when screencasting on MacOS
* Add support for uploading to more remote backends than just S3

