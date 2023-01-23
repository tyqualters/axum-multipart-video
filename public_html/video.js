!function() {
    const createElement = (tag, attributes) => {
        if(!(typeof tag == "string")) throw "Tag must be a string.";
        if(!(attributes instanceof Array)) throw "Attributes must be an array.";
        let element = window.document.createElement(tag);
        for(let attr of attributes) {
            if(!(attr instanceof Array)) throw "Attribute must be an array: ['AttrName', 'AttrValue'].";
            if(attr[0] !== undefined && !(typeof attr[0] == "string")) throw "Attribute name must be a string.";
            else if(attr[0] === undefined) throw "Attribute name cannot be empty.";
            if(attr[1] !== undefined && !(typeof attr[1] == "string")) throw "Attribute value must be a string.";
            element.setAttribute(attr[0], attr[1]);
        }
        return element;
    }

    window.document.querySelector('a#upload').addEventListener('click', (event) => {
        event.preventDefault();
        event.stopPropagation();
        
        let formElem = createElement('form', [['action', '/upload'], ['method', 'POST'], ['enctype', 'multipart/form-data']]);
        let fileBrowserBtn = createElement('input', [['type', 'file'], ['name', 'file'], ['accept', 'video/mp4']]);
        let uploadBtn = createElement('input', [['type', 'submit'], ['value', 'Upload video']]);

        formElem.appendChild(fileBrowserBtn);
        formElem.appendChild(uploadBtn);

        window.document.body.replaceChildren(formElem);
    });

    let params = new URLSearchParams(window.location.search);
    if(params.has('watch')) {
        let video_url = `${window.location.origin}/video/${params.get('watch')}.mp4`
        // Render the video as a <video>
        let video = createElement('video', [['width', '1280'], ['height', '720'], ['controls']]);
        let source = createElement('source', [['src', video_url], ['type', 'video/mp4']]);
        
        video.appendChild(source);
        
        window.document.body.replaceChildren(video);
    }
}();