package com.aziot.security.audit;

import com.aziot.dao.entity.system.SysAuditLog;
import com.aziot.dao.mapper.system.SysAuditLogMapper;
import jakarta.servlet.http.HttpServletRequest;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.annotation.Around;
import org.aspectj.lang.annotation.Aspect;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.stereotype.Component;
import org.springframework.web.context.request.RequestContextHolder;
import org.springframework.web.context.request.ServletRequestAttributes;

@Slf4j
@Aspect
@Component
@RequiredArgsConstructor
public class AuditLogAspect {

    private final SysAuditLogMapper auditLogMapper;

    @Around("@annotation(auditLog)")
    public Object around(ProceedingJoinPoint pjp, AuditLog auditLog) throws Throwable {
        long start = System.currentTimeMillis();
        SysAuditLog logEntry = new SysAuditLog();
        logEntry.setModule(auditLog.module());
        logEntry.setAction(auditLog.action());

        try {
            var auth = SecurityContextHolder.getContext().getAuthentication();
            if (auth != null) {
                logEntry.setOperatorName(auth.getName());
            }

            ServletRequestAttributes attrs = (ServletRequestAttributes) RequestContextHolder.getRequestAttributes();
            if (attrs != null) {
                HttpServletRequest req = attrs.getRequest();
                logEntry.setRequestIp(req.getRemoteAddr());
                logEntry.setRequestMethod(req.getMethod());
                logEntry.setRequestUrl(req.getRequestURI());
            }

            Object result = pjp.proceed();
            logEntry.setStatus("1");
            logEntry.setCostMs((int) (System.currentTimeMillis() - start));
            auditLogMapper.insert(logEntry);
            return result;
        } catch (Exception e) {
            logEntry.setStatus("0");
            logEntry.setCostMs((int) (System.currentTimeMillis() - start));
            auditLogMapper.insert(logEntry);
            throw e;
        }
    }
}
