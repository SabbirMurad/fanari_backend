class ToastNotification extends HTMLElement {
  static iconDefault = `
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" style="width:100%;">
    <style>circle{fill:var(--notification-background,#ffffff);}</style>
    <circle cx="50" cy="50" r="25"/>
  </svg>
`
  static iconSuccess = `
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" style="width:100%;">
    <style>rect{fill:var(--notification-background,#ffffff);}</style>
    <rect x="31" y="46" width="50" height="8" rx="4" ry="4" transform="translate(-18 54) rotate(-45)"/>
    <rect x="31" y="46" width="8" height="25" rx="4" ry="4" transform="translate(-30 42) rotate(-45)"/>
  </svg>
`

  static iconInfoWarn = `
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" style="width:100%;">
    <style>rect{fill:var(--notification-background,#ffffff);}</style>
    <rect x="46" y="25" width="8" height="34" rx="4" ry="4"/>
    <rect x="46" y="67" width="8" height="8" rx="4" ry="4"/>
  </svg>
`

  static iconError = `
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" style="width:100%;">
    <style>rect{fill:var(--notification-background,#ffffff);}</style>
    <rect x="46" y="25" width="8" height="50" rx="4" ry="4" transform="translate(-20 50) rotate(-45)"/>
    <rect x="25" y="46" width="50" height="8" rx="4" ry="4" transform="translate(-20 50) rotate(-45)"/>
  </svg>
`

  static crossIcon = `
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" style="width:100%;">
    <style>.cross-svg-circle{fill:none; stroke-miterlimit:10; stroke-width:6px; stroke-dasharray: 285;
      transform: rotate(-90deg) scale(1, -1);transform-origin: center;}</style>
    <g><path class="cross-svg-line" id="Path_185" data-name="Path 185" d="M66.48,62.15a3.08,3.08,0,0,1-4.24,4.46l-.11-.11L50,54.35,37.86,66.5a3.07,3.07,0,1,1-4.34-4.35L45.64,50,33.52,37.85a3.07,3.07,0,1,1,4.34-4.35L50,45.65,62.14,33.5a3.07,3.07,0,1,1,4.34,4.35h0L54.34,50Z"/></g>
    <circle class="cross-svg-circle" cx="50" cy="50" r="44.57"/>
  </svg>
  `

  #notifications = []
  #timeout = null;

  #position = "top-right"
  set position(value){
    let options = ['top-right','top-center','top-left','bottom-right','bottom-left','bottom-center']
    
    if(options.includes(value)){
      this.#position = value
    }

    this.shadow.appendChild(this.#render())
    if(this.#position === "bottom-left" || this.#position === "bottom-right" || this.#position === "bottom-center"){
      let notificationCount = this.shadow.querySelector('.notification-count')
      notificationCount.classList.add(`bottom`)
    }
  }

  //the amount of time after the which the notification will be automatically removed
  #timer = 5000
  set timer(value){
    this.#timer = value
  }

  constructor() {
    super()
    this.shadow = this.attachShadow({ mode: "closed" })
  }

  // the function you need to call from outside
  setNotification(notification) {
    this.#notifications.push(notification)
    let notificationCount = this.#notifications.length
    if(notificationCount > 0 && notificationCount < 4) {
      this.#renderNotification(notificationCount)
    }
    this.#changeCount()
  }

  //renders one notification, depending on the index from 0 to 2
  #renderNotification(index) {
    if(this.#notifications[index -1]){
      let wrapper = this.shadow.querySelector('.data-wrapper')
      wrapper.style.display = 'flex'

      let container = this.shadow.querySelector('.notification-container')

      let color = this.#getColor(this.#notifications[index -1].type)

      let notificationItem = document.createElement('div')
      notificationItem.classList.add('notification-item')
      notificationItem.classList.add(`notify-${index}`)
      if(this.#position === "bottom-left" || this.#position === "bottom-right" || this.#position === "bottom-center"){
        notificationItem.classList.add(`bottom`)
      }
      
      notificationItem.innerHTML = `
        <div class="notification-icon" style="background-color:${color};">
          ${this.#getIcon(this.#notifications[index - 1].type)}
        </div>
        <p class="notification-text">${this.#notifications[index - 1].message}</p>
        <div class="cross-btn"  onClick="this.getRootNode().host.closeByCross(event)">
          ${this.#getCrossIcon(this.#notifications[index - 1].type)}
        </div>
      `

      container.appendChild(notificationItem)
    }
    if(index === 1) {
      //comment this line out to stop the 1st auto remove
      this.#startCounter()
    }
  }

  //starts the counter for closing notification
  #startCounter() {
    let crossIcon = this.shadow.querySelector('.notification-item.notify-1 .cross-btn')
    let progressCircle = crossIcon.querySelector(".cross-svg-circle")

    let radius = progressCircle.r.baseVal.value
    let circumference = radius * 2 * Math.PI
    progressCircle.style.strokeDasharray = circumference

    let fraction = this.#timer / 100;

    let percent = 100;
    this.interVal = setInterval(() => {
      progressCircle.style.strokeDashoffset = circumference - (percent/100)*circumference;
      percent--;
    }, fraction)

    this.#timeout = setTimeout(() => {
      clearInterval(this.interval)
      let item = this.shadow.querySelector('.notification-item.notify-1')
      this.#closeNotification(item)

    }, this.#timer)
  }

  // returns the color needed for the type of notification
  #getColor(type){
    switch (type) {
      case "success":
        return "var(--notification-success,#71ff76)"
      case "error":
        return "var(--notification-error,#ff6868)"
      case "info":
        return "var(--notification-info,#6970ff)"
      case "default":
        return "var(--notification-default,#afafaf)"
      default:
        return "var(--notification-default,#afafaf)"
    }
  }
  //returns the correct icon based on the type
  #getIcon(type) {
    switch (type) {
      case "success":
        return ToastNotification.iconSuccess
      case "error":
        return ToastNotification.iconError
      case "info":
        return ToastNotification.iconInfoWarn
      case "default":
        return ToastNotification.iconDefault
      default:
        return ToastNotification.iconDefault
    }
  }

  // colors the cross icon to appropriate color depending on type and returns the icon
  #getCrossIcon(type) {
    let item = document.createElement('div')
    item.innerHTML = ToastNotification.crossIcon
    let circle = item.querySelector('.cross-svg-circle')
    let line = item.querySelector('.cross-svg-line')
    let color = this.#getColor(type)
    
    circle.style.stroke = color
    line.style.fill = color
    return item.innerHTML
  }

  // renders the initial boiler plate without the notifications
  #render() {
    let extra = this.#notifications.length - 3;

    let template = document.createElement("template")
    template.innerHTML = `
      <link rel="stylesheet" href="/components/toast-notification/toast-notification.css">
      <div class="data-wrapper ${this.#position}">
        <div class="notification-container">
        
        </div>
        <p class="notification-count">${extra > 0 ? `+ ${extra}` : ""}</p>
      </div>
    `;
    return template.content
  }

  //called when cross icon is pressed
  closeByCross(event) {
    let target = event.currentTarget
    let item = target.parentElement
    this.#closeNotification(item)
  }

  //closes the notification
  #closeNotification(item){

    item.classList.add('notification-remove')

    let secondItem = this.shadow.querySelector('.notification-item.notify-2')
    if(secondItem) {
      secondItem.classList.remove('notify-2')
      secondItem.classList.add('notify-1')
    }
    let thirdItem = this.shadow.querySelector('.notification-item.notify-3')
    if(thirdItem) {
      thirdItem.classList.remove('notify-3')
      thirdItem.classList.add('notify-2')
    }

    //removing the notification from array
    this.#notifications.shift()
    this.#renderNewNotification()
    this.#changeCount()

    setTimeout(() => {
      item.remove()
      clearInterval(this.interVal)
      clearTimeout(this.#timeout)
      if(this.#notifications.length != 0 ){
        this.#startCounter()
      }
      else {
        let wrapper = this.shadow.querySelector('.data-wrapper')
        wrapper.style.display = 'none'
      }
    }, 500)
  }

  //changes the count of extra notification available
  #changeCount() {
    let extra = this.#notifications.length - 3;
    let counter = this.shadow.querySelector('.notification-count')
    counter.innerHTML = `
    ${extra > 0 ? `+ ${extra}` : ""}
    `
  }

  //renders new notification after closing one if available
  #renderNewNotification() {
    // always checks for the third element, if does not exist then no need to render
    this.#renderNotification(3)
  }
}

export const toastNotification = {
  mount: function() {
    customElements.define("toast-notification", ToastNotification)
  },
}
