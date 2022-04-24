import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesCreateManyInput } from './devices-create-many.input';

@ArgsType()
export class CreateManyDevicesArgs {

    @Field(() => [DevicesCreateManyInput], {nullable:false})
    data!: Array<DevicesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
