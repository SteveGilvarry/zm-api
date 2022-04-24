import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsCreateManyInput } from './snapshots-create-many.input';

@ArgsType()
export class CreateManySnapshotsArgs {

    @Field(() => [SnapshotsCreateManyInput], {nullable:false})
    data!: Array<SnapshotsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
