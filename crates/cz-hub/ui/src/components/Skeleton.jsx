import React from 'react';

export const Skeleton = ({ className }) => (
    <div className={`skeleton ${className}`} />
);

export const CardSkeleton = () => (
    <div className="glass p-5 space-y-4">
        <div className="flex justify-between items-start">
            <Skeleton className="w-10 h-10 rounded-xl" />
            <Skeleton className="w-16 h-5 rounded-full" />
        </div>
        <div className="space-y-2">
            <Skeleton className="w-1/2 h-3" />
            <Skeleton className="w-3/4 h-6" />
        </div>
    </div>
);
