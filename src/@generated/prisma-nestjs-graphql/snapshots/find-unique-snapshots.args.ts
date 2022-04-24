import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereUniqueInput } from './snapshots-where-unique.input';

@ArgsType()
export class FindUniqueSnapshotsArgs {

    @Field(() => SnapshotsWhereUniqueInput, {nullable:false})
    where!: SnapshotsWhereUniqueInput;
}