class ImageUploaderVtwo extends HTMLElement {
  #imageData = undefined
  #preview
  #defaultImage = '/components/image-uploader-vtwo/icon/default.svg'

  get imageData() {
    return this.#imageData
  }

  constructor() {
    super()
    this.shadow = this.attachShadow({ mode: "closed" })
    this.shadow.appendChild(this.#render())
    this.#uploadByClick()
  }

  #render() {
    let template = document.createElement("template")
    template.innerHTML = `
      <link rel="stylesheet" href="/components/image-uploader-vtwo/image-uploader-vtwo.css">
      <div class="data-wrapper">
        <img
          class="upload-icon"
          src="${this.#defaultImage}"
          alt="preview-image"
          onerror="this.getRootNode().host.handleImageError(event)"
          data-image-loaded="false"
        >
        <input class="image_input" type="file" accept=".png,.gif" multiple>
      </div>
    `
    return template.content
  }

  handleImageError(event) {
    let target = event.currentTarget
    target.setAttribute('src', `${this.#defaultImage}`)
  }

  arrayBufferToBase64(buffer) {
    let binary = ''
    let bytes = new Uint8Array(buffer)
    let len = bytes.byteLength
    for (var i = 0; i < len; i++) {
      binary += String.fromCharCode(bytes[i])
    }
    return "data:image/png;base64," + window.btoa(binary)
  }

  #getImageSize(file) {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.src = URL.createObjectURL(file);

      img.onload = function () {
        resolve({ width: img.naturalWidth, height: img.naturalHeight });
        URL.revokeObjectURL(img.src); // Clean up
      };

      img.onerror = reject; // Handle errors
    });
  }

  #uploadByClick() {
    let myFile = this.shadow.querySelector('.image_input')

    myFile.addEventListener('change', async (e) => {
      let files = e.target.files

      this.#imageData = [];

      if (!files[0]) { return }
      for (let file of files) {
        let img_width, img_height;
        try {
          const { width, height } = await this.#getImageSize(file);
          img_width = width;
          img_height = height;
        } catch (error) {
          console.error(error);
          continue;
        }

        let mime = file.type.split('/')

        let buffer = await file.arrayBuffer()
        let payload = new Uint8Array(buffer)

        let blob = new Blob([payload], { 'type': mime })
        let url = URL.createObjectURL(blob)

        let name = file.name.split('.')
        name.pop()
        name = name.join('.')

        let reqBody = {
          name: name,
          size: file.size,
          height: img_height,
          width: img_width,
          type: mime[1].charAt(0).toUpperCase() + mime[1].slice(1),
          data_array: Array.from(payload),
          data_url: url
        }

        this.#imageData.push(reqBody)
      }

      this.dispatchEvent(new Event('change'))
    })
  }

  static get observedAttributes() { return ['default', 'preview'] }

  attributeChangedCallback(name, oldValue, newValue) {
    let preview = this.shadow.querySelector('.upload-icon')
    switch (name) {
      case 'default':
        this.#defaultImage = newValue;
        preview.setAttribute('src', newValue)
        break;
      case 'preview':
        preview.setAttribute('src', newValue)
        break;
      default:
        break;
    }
  }
}

export const imageUploaderVtwo = {
  mount: function () {
    customElements.define('image-uploader-vtwo', ImageUploaderVtwo)
  },
  unmount: function (index) {
    index
      ? document.querySelectorAll('image-uploader-vtwo')[index].remove()
      : document.querySelector('image-uploader-vtwo').remove()
  }
}