import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereUniqueInput } from './snapshots-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueSnapshotsArgs {

    @Field(() => SnapshotsWhereUniqueInput, {nullable:false})
    @Type(() => SnapshotsWhereUniqueInput)
    where!: SnapshotsWhereUniqueInput;
}
