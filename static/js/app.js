var IOBox = (function(el) {
    var _el = el;

    return {
        clear: function() { _el.innerHTML = ""; },
        log: function(str) { _el.innerHTML += "<br /><span>" + str + "</span>"; }
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

(function() {
    console.log("init tag UI");

    let uploader = Uploader();

    let iobox   = IOBox(document.querySelector("#io-box"));
    let tagger  = document.querySelector("#tagger");
    let dropbox = document.querySelector("#dropbox");


    iobox.clear();
    iobox.log("initialized io box ...");
    iobox.log("debug messages will show up here ...");

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


    console.log(tagger);
})();
