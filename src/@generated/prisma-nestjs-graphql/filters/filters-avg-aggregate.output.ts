import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Float } from '@nestjs/graphql';

@ObjectType()
export class FiltersAvgAggregate {

    @Field(() => Float, {nullable:true})
    Id?: number;

    @Field(() => Float, {nullable:true})
    UserId?: number;

    @Field(() => Float, {nullable:true})
    AutoArchive?: number;

    @Field(() => Float, {nullable:true})
    AutoUnarchive?: number;

    @Field(() => Float, {nullable:true})
    AutoVideo?: number;

    @Field(() => Float, {nullable:true})
    AutoUpload?: number;

    @Field(() => Float, {nullable:true})
    AutoEmail?: number;

    @Field(() => Float, {nullable:true})
    AutoMessage?: number;

    @Field(() => Float, {nullable:true})
    AutoExecute?: number;

    @Field(() => Float, {nullable:true})
    AutoDelete?: number;

    @Field(() => Float, {nullable:true})
    AutoMove?: number;

    @Field(() => Float, {nullable:true})
    AutoCopy?: number;

    @Field(() => Float, {nullable:true})
    AutoCopyTo?: number;

    @Field(() => Float, {nullable:true})
    AutoMoveTo?: number;

    @Field(() => Float, {nullable:true})
    UpdateDiskSpace?: number;

    @Field(() => Float, {nullable:true})
    Background?: number;

    @Field(() => Float, {nullable:true})
    Concurrent?: number;

    @Field(() => Float, {nullable:true})
    LockRows?: number;
}
