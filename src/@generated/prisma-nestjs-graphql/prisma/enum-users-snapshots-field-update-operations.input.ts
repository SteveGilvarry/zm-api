import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Snapshots } from './users-snapshots.enum';

@InputType()
export class EnumUsers_SnapshotsFieldUpdateOperationsInput {

    @Field(() => Users_Snapshots, {nullable:true})
    set?: keyof typeof Users_Snapshots;
}
