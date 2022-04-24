import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesCreateInput } from './devices-create.input';

@ArgsType()
export class CreateOneDevicesArgs {

    @Field(() => DevicesCreateInput, {nullable:false})
    data!: DevicesCreateInput;
}
