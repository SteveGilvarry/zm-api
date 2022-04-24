import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Events_Orientation } from '../events/events-orientation.enum';

@InputType()
export class EnumEvents_OrientationFieldUpdateOperationsInput {

    @Field(() => Events_Orientation, {nullable:true})
    set?: keyof typeof Events_Orientation;
}
