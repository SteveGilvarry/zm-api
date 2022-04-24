import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class FiltersSumAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    UserId?: true;

    @Field(() => Boolean, {nullable:true})
    AutoArchive?: true;

    @Field(() => Boolean, {nullable:true})
    AutoUnarchive?: true;

    @Field(() => Boolean, {nullable:true})
    AutoVideo?: true;

    @Field(() => Boolean, {nullable:true})
    AutoUpload?: true;

    @Field(() => Boolean, {nullable:true})
    AutoEmail?: true;

    @Field(() => Boolean, {nullable:true})
    AutoMessage?: true;

    @Field(() => Boolean, {nullable:true})
    AutoExecute?: true;

    @Field(() => Boolean, {nullable:true})
    AutoDelete?: true;

    @Field(() => Boolean, {nullable:true})
    AutoMove?: true;

    @Field(() => Boolean, {nullable:true})
    AutoCopy?: true;

    @Field(() => Boolean, {nullable:true})
    AutoCopyTo?: true;

    @Field(() => Boolean, {nullable:true})
    AutoMoveTo?: true;

    @Field(() => Boolean, {nullable:true})
    UpdateDiskSpace?: true;

    @Field(() => Boolean, {nullable:true})
    Background?: true;

    @Field(() => Boolean, {nullable:true})
    Concurrent?: true;

    @Field(() => Boolean, {nullable:true})
    LockRows?: true;
}
