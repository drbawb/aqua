var IOBox = (function(el) {
    var _el = el;

    return {
        clear: function() { if (_el) _el.innerHTML = ""; },
        log: function(str) { if (_el) _el.innerHTML += "<br /><span>" + str + "</span>"; }
    };  
});

var Uploader = (function() {
    var handleDropEvt = function(evt) {
        let transfer = evt.dataTransfer.items[0];
        if (!(transfer && transfer.kind == 'file')) { console.warn("u wot m8?"); return; }

        var fd = new FormData();
        fd.append("upload", transfer.getAsFile());

        var xhr = new XMLHttpRequest();
        xhr.open("POST", "/entries/upload", true);
        xhr.onreadystatechange = function() {
            if (xhr.readyState != 4) { return; }
            if (xhr.status != 200) { console.warn("ajax err!"); return; }

            console.log("ajax came back ^^,");
            console.log(xhr.responseText);
        };

        xhr.send(fd);

        // submit the entry to the server ...
        // POST /entries/remote {url}
        // POST /entries/upload {path}


        // handle the result
        // Submission :: {
        //  status  :: (Ok | Err),
        //  id      :: integer,
        //  hash    :: [u8],
        //  tags    :: [<string>, ...],
        // }
        //
        // the submission will point to the new entry, if it was a duplicate
        // then we get the current tagset which we can now modify...
        //

        // wait for XHR response
        // update tag list w/ submission->tags
        // apply future tagset changes to submission->id
    };


    return {
        handleDrop: handleDropEvt,
    };

});

var Gallery = (function() {
    let thumbs = document.querySelectorAll(".list.thumb");
    if (thumbs.length == 0) {
        console.log("no gallery!?");
        return;
    }

    console.log("binding list to " + thumbs.length + " entries");
    
    let lightbox      = document.querySelector("#light-box");
    let lightboxClose = lightbox.querySelector(".modal-close");
    let lightboxFull  = lightbox.querySelector(".modal-image");
    let lightboxImg   = lightbox.querySelector(".img-preview");
    let lightboxTags  = lightbox.querySelector(".img-tags");
   
    // let user close the lightbox  
    lightboxClose.addEventListener("click", function(evt) {
        lightbox.classList.remove("visible");
    });

    // bind click for thumbnail
    for (var i = 0; i < thumbs.length; i++) {
        let thumb = thumbs[i];

        thumb.addEventListener("click", function(evt) {
            evt.stopPropagation();
            evt.preventDefault();

            // clear lightbox
            lightboxTags.innerHTML = "Loading ...";
            lightboxImg.innerHTML  = "";

            let imgUrl  = "/entries/" + this.dataset.entryId;
            let tagsUrl = "/entries/" + this.dataset.entryId + "/tags";

            // load the image
            let img = document.createElement("img");
            img.src = imgUrl;
            img.style.width = "100%";
            img.style.height = "100%";
            lightboxImg.appendChild(img);

            // try to fetch tags
            let xhr = new XMLHttpRequest();
            xhr.open("GET", tagsUrl, true);
            xhr.addEventListener("load", function() {
                if (this.status != 200) {
                    console.warn(this.responseText);
                    return;
                }

                lightboxTags.innerHTML = this.responseText;
            });
            xhr.send();



            lightbox.classList.add("visible");

        });

    }
});



(function() {
    console.log("init tag UI");

    let gallery = Gallery();

    let uploader = Uploader();
    let iobox    = IOBox(document.querySelector("#io-box"));
    let tagger   = document.querySelector("#tagger");
    let dropbox  = document.querySelector("#dropbox");

    iobox.clear();
    iobox.log("initialized io box ...");
    iobox.log("debug messages will show up here ...");

    if (dropbox) {
        dropbox.addEventListener("dragenter", function(evt) {
            evt.preventDefault();
            console.log("drag enter");
            console.log(evt);
            dropbox.classList.add("hover");
        });

        dropbox.addEventListener("dragleave", function(evt) {
            evt.preventDefault();
            console.log("drag leave");
            console.log(evt);
            dropbox.classList.remove("hover");
        });

        dropbox.addEventListener("dragover", function(evt) {
            evt.preventDefault();
        });

        dropbox.addEventListener("dragend", function(evt) {
            evt.preventDefault();
            console.log("drag end");
            console.log(evt);

            dropbox.classList.remove("hover");
        });

        dropbox.addEventListener("drop", function(evt) {
            evt.preventDefault();
            console.log("drop");
            console.log(evt);
            dropbox.classList.remove("hover");

            uploader.handleDrop(evt);
        });
    }

    console.log(tagger);
})();
