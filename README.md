## Busser

#### Simple HTTPS website hosting in Rust with [Axum](https://github.com/tokio-rs/axum).

1. Just create a folder with your ```.html/.css/.js``` and other resources, ```.png, .gif, ...```
2. Point Busser to it with a config.json.
3. Run it, and that's it!*

\* you'll need certificates for https, and open ports if on a server

### Configuration

The ```config.json``` specifies key properties of the site and its content

```json
{
    "port_https": 443,
    "port_http": 80, 
    "throttle": {"max_requests_per_second": 512.0, "timeout_millis": 5000, "clear_period_seconds": 86400},
    "path": "pages",
    "home": "pages/index.html",
    "domain": "your.cool.domain",
    "allow_without_extension": true,
    "notification_endpoint": { "addr": "https://discord.com/api/webhooks/xxx/yyy" },
    "cert_path": "certs/cert.pem",
    "key_path": "certs/key.pem"
}
```
You'll also need to configure the statistics collection, the ipinfo_token is an optional ipinfo.io API token (50k requests for free per month)
```json
{
    "save_period_seconds": 86400,
    "path": "stats",
    "hit_cooloff_seconds": 3600,
    "clear_period_seconds": 604800,
    "ipinfo_token": "xxxxxxxxxxxxxx" 
}
```

✔️ Host HTML/css/js/text content from a given directory 

✔️ Serve with and without ```.html```, e.g. ```/x/y/z/webpage.html``` via ```/x/y/z/webpage```

✔️ Host Image/video content (png, jpg, gif, webp, mp4, ...)

✔️ Http redirect to https

✔️ Https certificates

✔️ IP throttling

✔️ Hit statistics (with optional ipinfo.io integration)

✔️ Host via **free tier** cloud services!

🏗️ Discord webhook integration for status messages

____

#### GDPR, Cookie Policies, and Privacy Policies

Please be aware the following data is store/processed but is configurable for auto-deletetion

Currently the IP throttler stores for each IP a hit count/last request time, i.e

```rust
pub struct Requests
{
    count: u32,
    last_request_time: Instant,
    timeout: bool
}
```

The statistics collection stores the IP as well, if you use the optional ipfino.io integration the the data will also include e.g. country code and lattidue/longitude etc, that may be considered identifiable information.
____

### Free static website hosting example with Google Cloud Free Tier

The [gcloud free tier](https://cloud.google.com/free?hl=en) [allows for the following instance running 24/7:](https://cloud.google.com/free/docs/free-cloud-features#compute)

```
    1 non-preemptible e2-micro VM instance per month in one of the following US regions:
        Oregon: us-west1
        Iowa: us-central1
        South Carolina: us-east1
    30 GB-months standard persistent disk
    1 GB of outbound data transfer from North America to all region destinations (excluding China and Australia) per month

```

You will still see costs in the Google cloud console, or savings suggestions. You should recieve free tier discount deductions to nullify these cost completely

This can be verified by:

1. Navigating to the [Google Cloud Console](https://console.cloud.google.com)
2. Selecting ```Billing``` form the burger menu (top left as of now)
3. Selecting ```Cost Table```
4. In ```Filters``` (right) select ```SKUs```
5. Type ```e2```
6. Select the all (e.g. ```16 filtered results```)
7. Toggle the arrow for you project
8. Toggle ```Compute Engine```
9. You should see e.g. ```E2 Instance Core running in Americas``` with ```X.XX```
10. You should also see e.g.  ```E2 Instance Ram running with free tier discount``` with ```-X.XX```

As this is a website you may be charged for network costs over the 1 GB outbound data, or for traffic from China and Australia

##### Create it using the CLI...

Using the gloud cli this command should create an instance template for the free tier, which can be used to create instances

```bash
gcloud beta compute instance-templates create free-tier-template-http --project=YOUR_PROJECT --machine-type=e2-micro \\
--network-interface=network=default,network-tier=PREMIUM \\
--instance-template-region=projects/YOUR_PROJECT/regions/us-central1 --maintenance-policy=MIGRATE \\
--provisioning-model=STANDARD --service-account=YOUR_SERVICE_ACCOUNT \\
--scopes=https://www.googleapis.com/auth/devstorage.read_only,https://www.googleapis.com/auth/logging.write,https://www.googleapis.com/auth/monitoring.write,https://www.googleapis.com/auth/servicecontrol,https://www.googleapis.com/auth/service.management.readonly,https://www.googleapis.com/auth/trace.append \\
--enable-display-device --tags=http-server,https-server \\
--create-disk=auto-delete=yes,boot=yes,device-name=free-tier-template,image=projects/debian-cloud/global/images/debian-11-bullseye-v20220719,mode=rw,size=30,type=pd-standard 
--no-shielded-secure-boot --shielded-vtpm --shielded-integrity-monitoring --reservation-affinity=any
```

##### ...or using Cloud console

- create an e2 in us-central1 (Iowa) for both zone and region
- select e2-micro (0.25-2 vCPU 1GB memory)
- you can change the boot disc from 10GB to 30GB if you like
- allow HTTPS and HTTP (if you need it for certificate provising)
- all else as default

#### Network

Make sure 443 and 80 are open ports (or whatever ports you wish to serve on)

### https certificate setup

#### Self signed (useful for localhost testing)

- You can use the bash script ```certs/gen.sh``` to generate a key/cert pair with openssl

#### Production; from authority

- get a domain (e.g. from squarespace)
- create a custom DNS record, e.g.
    - ```your.domain.somwhere    A	1 hour	google.cloud.instance.ip ```
- Use [Let's Encrypts](https://letsencrypt.org/) recommendation of [certbot](https://certbot.eff.org/) it really is very easy
    - You will need to enable http in the cloud instance firewall for provisioning as well as https
